# SIM-1 Implementation Plan — Commander castable from the command zone (`StubProvider`)

**Generated**: 2026-08-02
**Task**: `scutemob-175` · **Worktree**: `/home/skydude/projects/scutemob/.worktrees/scutemob-175`
**Seed**: playtest triage **F7** (`memory/playtest-triage-2026-08-02.md:112-120`)
**Scope**: `crates/simulator/` production code ONLY. **Zero engine lines.**
**CR**: 903.6, 903.8, 903.9a/b, 408.1, 601.2 / 601.2a / 601.2b / 601.2f, 117.1a, 101.2
**Wire**: PROTOCOL **33** / HASH **70** must be UNMOVED (no new `Command`/`GameEvent`/`Effect` variant)

> **This is not a PB batch.** No new DSL primitive, no `Effect`/`Condition`/`TargetFilter`
> variant, no card defs. It closes a provider-vs-engine capability gap: the engine has
> supported command-zone casting since M6; `StubProvider` has never offered it.

---

## 0. Premise — re-verified on this branch, by symbol

Every fact in the dispatch brief was re-read in this worktree. All hold. Two facts the
brief did **not** state were found and are load-bearing (§0.2, §0.3).

### 0.1 The brief's facts, confirmed

| Claim | Site | Status |
|---|---|---|
| provider enumerates casts from hand only | `legal_actions.rs:524-593` (`let hand = ZoneId::Hand(player)` at `:526`) | CONFIRMED |
| `ZoneId::Command` appears only for the CR 903.9a return | `legal_actions.rs:369-377` | CONFIRMED |
| setup puts the commander in the command zone **and** registers it | `setup.rs:272-276` (`.in_zone(ZoneId::Command(pid))` + `builder.player_commander(pid, ..)`) | CONFIRMED |
| zone-derived command-zone detection | `casting.rs:255` | CONFIRMED |
| the "not in your hand" gate accepts it | `casting.rs:714-743` | CONFIRMED |
| CR 903.8 gate keys on **`CardId`**, not `ObjectId` | `casting.rs:760-772` (`player_state.commander_ids.contains(cid)`) | CONFIRMED |
| tax applied on top of base cost | `casting.rs:2655-2667` | CONFIRMED |
| `apply_commander_tax` adds `tax*2` to `generic` only, saturating | `commander.rs:328-342` | CONFIRMED |
| it is `pub` and re-exported | `crates/engine/src/lib.rs:19` | CONFIRMED |
| tax counter increments on cast | `casting.rs:4701-4714` | CONFIRMED |
| `from_zone` is never read anywhere | workspace-wide grep: constructed at `legal_actions.rs:587`, declared at `:25`, read **nowhere** | CONFIRMED |
| `params.rs` forwards the bare card | `params.rs:255-258` (`LegalAction::CastSpell { card, .. } => CastSpellData { card: *card, .. }`) | CONFIRMED |

Two more, needed by the design:

* **`PlayerState::commander_ids: Vector<CardId>` and `commander_tax: OrdMap<CardId, u32>` are
  both `pub`** (`crates/card-types/src/state/player.rs:373`, `:383`), so `crates/simulator`
  can read them without any engine change.
* **The engine's sorcery-speed gate is zone-agnostic** (`casting.rs:3471-3485`): it keys on
  `is_instant_speed` derived from layer-resolved characteristics plus
  `has_active_flash_grant`, and knows nothing about which zone the card came from. So a
  commander cast obeys CR 117.1a exactly as a hand cast does, and the provider must mirror
  the hand loop's timing logic rather than hard-code "sorcery speed".

### 0.2 NEW FINDING (blocking) — Drannith Magistrate becomes reachable

`casting.rs:6720-6732`:

```rust
GameRestriction::OpponentsCantCastFromNonHand => {
    if player != controller {
        if let Some(zone) = card_zone {
            if zone != ZoneId::Hand(player) { return Err(...); }
        }
    }
}
```

`legal_actions.rs::is_cast_restricted_by_stax` says in its own doc comment (`:1596-1598`)
that it **does not** check per-card zone restrictions "like Drannith Magistrate's zone
restriction". That has been harmless precisely because the provider only ever offered
**hand** casts, and a hand cast always satisfies `zone == Hand(player)`.

The moment SIM-1 offers a command-zone cast, **every** such offer is a non-hand cast, so an
opponent's Drannith Magistrate makes the engine reject **100%** of them. `drannith_magistrate.rs`
uses `..Default::default()` and `Completeness`'s `#[default]` is `Complete`
(`crates/card-types/src/cards/card_definition.rs:197-200`), so the card is **deck-legal** and
appears in `DeckSource::RandomPerSeat` games.

**Therefore SIM-1 must add a Drannith check to the command-zone loop.** Omitting it would
ship a brand-new, guaranteed SR-38 violation ("never offer an action the engine rejects").
See Step 4.

*Not in scope*: `MaxNoncreatureSpellsPerTurn` / `MaxNonartifactSpellsPerTurn` are also
unchecked by the provider, for hand casts too. That asymmetry is pre-existing and unchanged
by SIM-1; it is recorded as **OOS-SIM1-3** rather than silently widened.

### 0.3 NEW FINDING (de-risking) — the fuzzer's games have no commanders

`crates/simulator/src/bin/fuzzer.rs:320-328` puts the commander card object into
`ZoneId::Command(pid)` but **never calls `builder.player_commander(..)`**. So
`PlayerState::commander_ids` is **empty** in every fuzzer game. `crates/simulator/tests/local_game.rs::build_state`
(`:74-90`) has the identical shape.

This is exactly the gap `setup.rs:258-270` documents ("A game built with the object but not
the registration is not a Commander game: the commander is recastable for free forever and
deals no commander damage").

**Consequence for SIM-1**: because the offer is gated on `commander_ids` (mirroring
`casting.rs:760-772`), **no fuzzer game and no `tests/local_game.rs` fixture can produce the
new action at all.** Recorded fuzz seeds and the whole of `tests/local_game.rs` are immune by
construction, not by luck. §7 turns this into an assertion rather than a claim.

File the fuzzer's own gap as **OOS-SIM1-4** (deliberately unfixed here: making the fuzzer a
real Commander game would move every recorded seed, which is precisely what this batch is
trying not to do).

---

## 1. CR rule text (MCP, verbatim)

**903.6** — At the start of the game, each player puts their commander from their deck face
up into the command zone. Then each player shuffles the remaining cards of their deck so that
the cards are in a random order. Those cards become the player's library.

**903.8** — A player may cast a commander they own from the command zone. A commander cast
from the command zone costs an additional {2} for each previous time the player casting it has
cast it from the command zone that game. This additional cost is informally known as the
"commander tax."

**408.1** — The command zone is a game area reserved for certain specialized objects that have
an overarching effect on the game, yet are not permanents and cannot be destroyed.

**903.9a** — If a commander is in a graveyard or in exile and that object was put into that
zone since the last time state-based actions were checked, its owner may put it into the
command zone. This is a state-based action.

**903.9b** — If a commander would be put into its owner's hand or library from anywhere, its
owner may put it into the command zone instead. …

**601.2** — To cast a spell is to take it from where it is (**usually** the hand), put it on
the stack, and pay its costs … Casting a spell includes proposal of the spell (rules
601.2a–d) and determination and payment of costs (rules 601.2f–h).

**601.2a** — To propose the casting of a spell, a player first moves that card … **from where
it is** to the stack. …

**601.2b** — … If the spell has a variable cost that will be paid as it's being cast (such as
an {X} in its mana cost; see rule 107.3), the player announces the value of that variable. …
If a cost that will be paid as the spell is being cast includes hybrid mana symbols, the
player announces the nonhybrid equivalent cost they intend to pay. If a cost … includes
Phyrexian mana symbols, the player announces whether they intend to pay 2 life or a
corresponding colored mana cost for each of those symbols. …

**601.2f** — The player determines the total cost of the spell. Usually this is just the mana
cost. Some spells have additional or alternative costs. … The total cost is the mana cost or
alternative cost (as determined in rule 601.2b), plus **all additional costs and cost
increases**, and minus all cost reductions. … Once the total cost is determined … the
resulting total cost becomes "locked in."

**117.1a** — A player may cast an instant spell any time they have priority. A player may cast
a noninstant spell during their main phase any time they have priority and the stack is empty.

**Reading for the engine**: 601.2 + 601.2a say "where it is" is not necessarily the hand —
903.8 is the permission for the command zone; 601.2f says the tax is a **cost increase folded
into the total cost**, which is exactly `apply_commander_tax` applied on top of the base cost
at `casting.rs:2664`. 117.1a gives the timing, which is the same timing the hand loop already
computes. 408.1 + 903.9a/b are why the command zone is **not** a "commanders only" zone: the
`commander_ids` filter, not the zone, is what keeps the provider a subset of the engine.

---

## 2. Design

### 2.1 The one shared helper

The tax must be known in **three** places (the brief's framing, confirmed by grep — the only
callers of `mana_solver::solve_mana_payment` in `crates/simulator` are these three):

| # | Site | Symbol | Today reads |
|---|---|---|---|
| 1 | offer gate | `legal_actions.rs::can_afford` at `:1525`, called from the cast loop at `:584` | printed `obj.characteristics.mana_cost` |
| 2 | **human** submit auto-tap | `local_game.rs::auto_tap_commands_for` `:607-627` | printed cost (`:612`), + `x_value*x_count` (`:614-616`) |
| 3 | **bot** auto-tap in `advance()` | `local_game.rs:438-454` | printed cost (`:440`), `unwrap_or_default()` |

Site 2's own doc block already names this defect in as many words (`local_game.rs:581-591`):
*"Recasting a taxed commander with a pool that covers only the printed cost therefore skips
tapping and the cast is rejected."*

**Decision: one shared helper, consumed by all three.** Not three local edits.

```rust
/// CR 903.8 / CR 601.2f: the mana cost this player will actually be charged for casting
/// `card` right now — the printed cost, plus commander tax when (and only when) `card` is
/// in `ZoneId::Command(player)` AND its `CardId` is one of `player`'s `commander_ids`.
///
/// Mirrors `rules/casting.rs`'s own two-part derivation exactly:
///   * `casting.rs:255`  — `casting_from_command_zone = card_obj.zone == ZoneId::Command(player)`
///   * `casting.rs:760`  — CR 903.8 gate on `player_state.commander_ids.contains(card_id)`
///   * `casting.rs:2656` — `apply_commander_tax(&base, tax)` where `tax` is the count of
///                         PREVIOUS casts (`commander_tax.get(cid)`, defaulting to 0).
///
/// The tax itself is **consumed from the engine** (`mtg_engine::apply_commander_tax`), never
/// re-derived here — SR-38's "only offer what the engine accepts" is only true if the two
/// arithmetics are literally the same function. Contrast `multiply_mana_cost`
/// (`legal_actions.rs:1480`), which is a *necessary* duplicate because the engine's copy is
/// private; this one is not, so duplicating it would be a choice and the wrong one.
///
/// Returns `None` when the object has no mana cost (an emblem in the command zone under
/// CR 408.1, a land, a missing object), which every caller treats as "nothing to pay for /
/// nothing to offer".
///
/// **Identity for every non-commander cast**: for a card in hand the two guards both fail
/// and the printed cost is returned unchanged, so no existing offer, plan or seed moves.
pub fn effective_cast_cost(
    state: &GameState,
    player: PlayerId,
    card: ObjectId,
) -> Option<ManaCost>
```

**Placement**: `crates/simulator/src/legal_actions.rs`, immediately above `can_afford`
(`:1502`), and `pub` + re-exported from `crates/simulator/src/lib.rs` next to
`pub use mana_solver::solve_mana_payment;` (`lib.rs:39`).

*Why `pub` and not `pub(crate)`*: there is a **fourth** printed-cost auto-tap site outside
this crate — `tools/tui/src/play/app.rs:280-294` (the TUI's bot path). SIM-1 does not fix it
(criterion 5987), but exporting the helper makes **OOS-SIM1-2** a one-line fix later instead
of a copy. `pub(crate)` is an acceptable alternative if the reviewer prefers minimal surface;
say which was chosen and why in the commit body either way.

### 2.2 Affordability — what the taxed cost does to hybrid / Phyrexian / {X}

**The helper returns the UNFLATTENED taxed cost.** Each caller then flattens exactly as it
already does. This is safe because of a commutation property worth pinning as a test:

> `apply_commander_tax` writes **only** `generic` (`commander.rs:330-341`: everything else is
> a field-for-field clone, including `hybrid`, `phyrexian`, `x_count`).
> `ManaCost::flatten_hybrid_phyrexian` **reads** `hybrid`/`phyrexian` and **adds** to
> colored/generic; it never reads `generic`. `auto_tap_commands_for` adds `x_value * x_count`
> to `generic` and never reads it either.
>
> Therefore `flatten(tax(c)) == tax(flatten(c))` and
> `tax(c).mana_value() == c.mana_value() + 2*tax`, and the order of tax / flatten / X-add is
> immaterial. **Test T11 pins this**, so a future "simplification" that flattens first cannot
> silently change what is charged.

Per-caller consequences:

* **Offer gate (site 1)** — call `can_afford(state, player, &effective)` exactly as the hand
  loop calls it today. Do **not** upgrade this call to `resolve_hybrid_phyrexian_plan`:
  `LegalAction::CastSpell` carries no hybrid/Phyrexian plan channel (the "KNOWN GAP" comment
  at `params.rs:277-279`), so the engine will flatten with the all-default plan and there is
  nothing for the provider to announce. A **hybrid-pipped commander** is therefore subject to
  exactly the same `can_afford` imprecision as a hybrid-pipped hand spell is today — a
  pre-existing class, not a new one. Record it as **OOS-SIM1-1** and do not widen the fix.
* **{X} commander** — `apply_commander_tax` preserves `x_count`. `can_afford` treats X as 0
  (`mana_solver.rs:253`: "x_count: X defaults to 0 for the solver"), unchanged from today.
  `auto_tap_commands_for` adds `x_value * x_count` on top of the taxed generic, which is the
  correct CR 601.2b→601.2f order (announce X, then total the cost including increases).
* **Phyrexian commander** — unchanged: the default plan pays mana, and life is never part of
  the tax.

### 2.3 Where the enumeration goes, and why the index order matters

Insert the command-zone loop **immediately after** the hand loop (i.e. after
`legal_actions.rs:593`), inside the same `if !cast_restricted` guard or as a sibling block.

* It must come **after** the `state.blocking_decision()` early return (`:299-366`), the CR
  903.9a commander-zone-choice early return (`:369-377`), the mulligan early return
  (`:379-384`) and the `priority_holder != Some(player)` early return (`:387-389`). Placing it
  after the hand loop satisfies all four.
* **Appending after the hand loop keeps every existing action's index unchanged.** `RandomBot`
  picks by index into this list, so prepending would reshuffle every bot choice in every
  seeded game for no reason. Appending confines the perturbation to games where a commander
  cast is actually offered.

### 2.4 What the loop must mirror from the hand loop, and what it must not

| Hand-loop element | Command-zone loop | Reason |
|---|---|---|
| `is_land` skip (`:528-531`) | **KEEP** | CR 117.1a: a land is played, not cast. No commander is a land today, but offering `CastSpell` for one would be an SR-38 violation the moment one exists. Cheap, symmetric, defensive. |
| `is_instant` / `has_flash` / flash-grant scan (`:533-572`) | **MIRROR EXACTLY** | The engine's timing gate (`casting.rs:3471-3485`) is zone-agnostic. A commander with Flash, or any commander under a `FlashGrantFilter::AllSpells` grant (Vedalken Orrery / Leyline of Anticipation), is legally castable at instant speed and must be offered. Hard-coding sorcery speed would **under**-offer. |
| `is_main_phase && stack_empty && is_active` fallback (`:578`) | **MIRROR** | CR 117.1a. |
| `can_afford(..)` on the object's `mana_cost` (`:583-584`) | **REPLACE** with `effective_cast_cost` | CR 903.8 / 601.2f. |
| `cast_restricted` (`:522`, `:525`) | **REUSE** | `MaxSpellsPerTurn`, `OpponentsCantCast*` — all zone-independent. |
| — | **ADD** the Drannith check (§0.2) | CR 101.2. New because the class is newly reachable. |
| — | **ADD** the `commander_ids` filter | CR 903.8 / 408.1. The zone can hold emblems and CR 903.9a returns; the filter is what keeps the offer a subset of the engine's `casting.rs:760-772`. |

**Recommended refactor** (optional but preferred): extract the timing block into

```rust
/// CR 117.1a / CR 601.3b: may `player` begin casting `obj` in the current window?
/// Shared by the hand and command-zone enumerations so the two cannot drift.
fn can_cast_at_this_time(
    state: &GameState,
    player: PlayerId,
    obj: &GameObject,
    is_main_phase: bool,
    stack_empty: bool,
    is_active: bool,
) -> bool
```

and call it from both loops. Behaviour-preserving for the hand loop (verify by the existing
suite staying green). If the runner prefers not to touch the hand loop at all, duplicating the
block is acceptable **only** with a comment naming the hand loop as its twin — but the
extraction is the better answer and this plan recommends it.

---

## 3. Ordered implementation steps

Every path is relative to `/home/skydude/projects/scutemob/.worktrees/scutemob-175/`.
Line numbers are as of this branch; **re-locate by symbol before editing** (the OOS-DP6-8
line-cite class).

### Step 1 — `effective_cast_cost`
**File**: `crates/simulator/src/legal_actions.rs`, insert immediately above `can_afford` (~`:1502`).
**Action**: add the function of §2.1. Body:
1. `let obj = state.objects().get(&card)?;` (or `state.object(card).ok()?`)
2. `let printed = obj.characteristics.mana_cost.clone()?;`
3. `if obj.zone != ZoneId::Command(player) { return Some(printed); }`
4. `let ps = state.player(player).ok()?;`
5. `let cid = obj.card_id.as_ref()?;`
6. `if !ps.commander_ids.contains(cid) { return Some(printed); }`
7. `let tax = ps.commander_tax.get(cid).copied().unwrap_or(0);`
8. `Some(mtg_engine::apply_commander_tax(&printed, tax))`

**Imports**: `apply_commander_tax` is re-exported at `crates/engine/src/lib.rs:19`; add it to
the `use mtg_engine::{..}` block at `legal_actions.rs:8-13` (or call it fully qualified, as
this file already does for `mtg_engine::rules::casting::can_pay_cost`). `ManaCost` and
`ObjectId` are already imported.

**Do NOT** write `generic + 2*tax` inline. Consuming `apply_commander_tax` is the point.

### Step 2 — export it
**File**: `crates/simulator/src/lib.rs`, near `:39`.
**Action**: `pub use legal_actions::effective_cast_cost;` (matching the existing
`pub use mana_solver::solve_mana_payment;` style).

### Step 3 — extract the timing predicate (recommended)
**File**: `crates/simulator/src/legal_actions.rs`.
**Action**: lift `:533-579` (is_instant / has_flash / flash-grant scan / `can_cast`) into
`can_cast_at_this_time` per §2.4; rewrite the hand loop to call it. **Behaviour must be
identical** — this is a pure extraction. Verify: the whole existing suite green with no test
edits.

### Step 4 — the Drannith mirror
**File**: `crates/simulator/src/legal_actions.rs`, next to `is_cast_restricted_by_stax` (~`:1599`).
**Action**: add

```rust
/// CR 101.2 (Drannith Magistrate): mirrors the `OpponentsCantCastFromNonHand` arm of
/// `rules/casting.rs::check_cast_restrictions` (`casting.rs:6720-6732`).
///
/// `is_cast_restricted_by_stax` deliberately does not check per-card ZONE restrictions —
/// its own doc says so — and that was harmless while the provider offered hand casts
/// only, because a hand cast always satisfies `zone == Hand(player)`. Every command-zone
/// cast is a non-hand cast, so without this check SIM-1 would offer an action the engine
/// rejects 100% of the time whenever any opponent controls a Drannith Magistrate
/// (`drannith_magistrate.rs`, `Completeness::Complete`, deck-legal). SR-38.
///
/// Player-level, not per-card: the engine's arm reduces to `zone != Hand(player)`, which
/// is unconditionally true for the command zone.
fn is_cast_from_nonhand_restricted(state: &GameState, player: PlayerId) -> bool
```

Body: iterate `state.restrictions()`, `continue` if the restriction's `source` is not on the
battlefield (same guard as `is_cast_restricted_by_stax:1605-1613`), and return `true` on
`GameRestriction::OpponentsCantCastFromNonHand` when `player != restriction.controller`.
`GameRestriction` is already imported (`:10`).

**Also update** `is_cast_restricted_by_stax`'s doc comment (`:1596-1598`) so the "does NOT
check per-card restrictions (like Drannith Magistrate's zone restriction)" sentence points at
the new function instead of describing an unhandled gap. A stale doc that says a gap is open
after it has been closed is the lying-cite class this project keeps finding.

### Step 5 — the command-zone enumeration
**File**: `crates/simulator/src/legal_actions.rs`, immediately after the hand loop (~`:593`).
**Action**:

```rust
// CR 903.8 / CR 601.2a (SIM-1, playtest triage F7): a player may cast a commander they
// OWN from the command zone. The engine has always supported this (`casting.rs:255`
// derives command-zone-ness from the object's zone; `:714` admits it; `:760` gates it on
// CR 903.8; `:2656` applies the tax; `:4702` increments it) — the provider simply never
// looked in the zone, so a human clicking their commander in the browser was told the
// server offered nothing.
//
// Three filters, each mirroring an engine gate rather than a preference:
//   * ZoneId::Command(PLAYER) only — never another seat's zone (`casting.rs:255`).
//   * `commander_ids` (CR 903.8, `casting.rs:760-772`) — CR 408.1 makes the command zone a
//     home for other objects too (emblems), and CR 903.9a/b can move things through it.
//     The zone is NOT the filter; `commander_ids` is.
//   * CR 101.2 non-hand cast restriction (`is_cast_from_nonhand_restricted`) — newly
//     reachable, see that function's doc.
//
// Timing is MIRRORED, not assumed: a commander is a permanent so it is normally sorcery
// speed (CR 117.1a), but the engine's own gate (`casting.rs:3471-3485`) is zone-agnostic,
// so a commander with Flash or under a CR 601.3b flash grant is legal at instant speed and
// must be offered.
//
// Appended AFTER the hand loop on purpose: `RandomBot` chooses by index, so appending
// leaves every pre-existing action's index untouched.
if !cast_restricted && !is_cast_from_nonhand_restricted(state, player) {
    let command_zone = ZoneId::Command(player);
    for obj in state.objects_in_zone(&command_zone) {
        // CR 117.1a: a land is played, not cast. No commander is a land today; the skip
        // mirrors the hand loop so the two cannot diverge if one ever is.
        if obj.characteristics.card_types.contains(&CardType::Land) { continue; }
        // CR 903.8 (`casting.rs:760-772`), keyed on CardId — NOT ObjectId.
        let Some(cid) = obj.card_id.as_ref() else { continue; };
        let is_commander = state.player(player)
            .map(|ps| ps.commander_ids.contains(cid))
            .unwrap_or(false);
        if !is_commander { continue; }
        if !can_cast_at_this_time(state, player, obj, is_main_phase, stack_empty, is_active) {
            continue;
        }
        // CR 903.8 / 601.2f: the tax is a cost INCREASE folded into the total cost, so
        // the affordability gate must see it or the offer 422s (SR-38).
        let Some(cost) = effective_cast_cost(state, player, obj.id) else { continue; };
        if can_afford(state, player, &cost) {
            actions.push(LegalAction::CastSpell { card: obj.id, from_zone: command_zone });
        }
    }
}
```

`from_zone: command_zone` is set for honesty; the field is read nowhere (§0.1) and no wire
type changes.

Adjust to whatever `objects_in_zone` actually yields (`&GameObject` vs owned) — the hand loop
at `:512` and `:527` is the template.

### Step 6 — human auto-tap
**File**: `crates/simulator/src/local_game.rs::auto_tap_commands_for`, `:607-627`.
**Action**: replace `:612`

```rust
let obj = self.state.object(cast.card).ok()?;
let mut cost = obj.characteristics.mana_cost.clone()?;
```

with

```rust
// CR 903.8 / CR 601.2f (SIM-1): the PRINTED cost is not what the engine charges. The
// shared helper applies commander tax when the card is being cast from this player's
// command zone, so the offer gate (`legal_actions::can_afford`), this human auto-tap and
// the bot auto-tap in `advance()` cannot disagree about what has to be paid.
let mut cost = legal_actions::effective_cast_cost(&self.state, player, cast.card)?;
```

Everything below is unchanged: the `x_value * x_count` add (`:614-616`) still applies on top
(§2.2 commutation), the pool early return (`:623-625`) now compares against the **taxed** cost
— which is the whole point — and `solve_mana_payment` (`:626`) is handed the taxed cost.

Note the removed `obj` binding: the function's own doc block (`:559-570`) enumerates "exactly
three" error-discarding `?`s and names `state.object(..).ok()?` as one of them. That one moves
*inside* the helper and still means "prepend no tapping commands", so the paragraph's claim
survives — but **update the count and the list**, do not leave it describing three `?`s at
sites that no longer exist.

**Also rewrite** the "Known limitation, and the other half of OOS-M11-2" paragraph
(`:581-591`). Its commander-tax sentence is now false. Keep the Thalia-style-increase,
cost-reduction and CR 106.12 `SpellContext` halves — those are still open — and say plainly
that the commander-tax half was closed by SIM-1.

### Step 7 — bot auto-tap
**File**: `crates/simulator/src/local_game.rs::advance`, `:438-454`.
**Action**: replace the `if let Ok(obj) = self.state.object(cast.card) { if let Some(ref cost) = obj.characteristics.mana_cost {` nest with a single

```rust
// CR 903.8 (SIM-1): same helper as the human path and the offer gate — see
// `auto_tap_commands_for`. A bot offered a taxed commander cast and handed a
// printed-cost tap plan gets its cast rejected, falls through to the `PassPriority`
// fallback below, and is re-offered the identical action next priority: `HeuristicBot`
// scores `CastSpell` at `50 + 10*mana_value` (`heuristic_bot.rs:180-192`) and
// `RepeatKey::of` returns `None` for `CastSpell` (`:52-63`), so nothing caps the retry.
let commands = if let Command::CastSpell(cast) = &cmd {
    match legal_actions::effective_cast_cost(&self.state, cast.player, cast.card) {
        Some(cost) => {
            let mut cmds = mana_solver::solve_mana_payment(&self.state, cast.player, &cost)
                .unwrap_or_default();
            cmds.push(cmd.clone());
            cmds
        }
        None => vec![cmd.clone()],
    }
} else {
    vec![cmd.clone()]
};
```

Preserve the existing `unwrap_or_default()` semantics (bot path deliberately proceeds and lets
the engine refuse) and the "no pool check by design" rationale at `:554-557`.

### Step 8 — doc/README truth-up
**Files**: `docs/mtg-engine-simulator.md`, `tools/play-server/README.md`.
**Action**: if either states that casts are enumerated from hand only, or lists "cannot cast
your commander" as a limitation, correct it and bump `<!-- last_updated: -->` if managed.
Grep both for `from hand` / `command zone` before editing; do not invent a limitation entry
that is not there.

### Step 9 — seeds
Append to `memory/workstream-state.md` (SIM-1 handoff) and, if the project files seeds
elsewhere, mirror there:

* **OOS-SIM1-1** — a hybrid/Phyrexian-pipped commander is gated by `can_afford`, not by
  `resolve_hybrid_phyrexian_plan`, because `LegalAction::CastSpell` has no plan channel
  (`params.rs:277-279`). Same imprecision every hybrid hand spell already has. Closing it means
  giving `CastSpell` the RS2 channel, which is a wire-adjacent PB, not SIM-1.
* **OOS-SIM1-2** — `tools/tui/src/play/app.rs:280-294` is a **fourth** printed-cost auto-tap
  site, outside `crates/simulator`. Its human path (`input.rs:158-178`) enumerates
  `app.hand_objects()` so a TUI human still cannot select a commander at all; its **bot** path
  will now be offered taxed commander casts and will plan the printed cost for them. One-line
  fix once `effective_cast_cost` is `pub`; out of scope by criterion 5987.
* **OOS-SIM1-3** — the provider mirrors `MaxSpellsPerTurn` / `OpponentsCantCast*` /
  `OpponentsCanOnlyCastAtSorcerySpeed` / (now) `OpponentsCantCastFromNonHand`, but **not**
  `MaxNoncreatureSpellsPerTurn` or `MaxNonartifactSpellsPerTurn` — pre-existing for hand casts,
  unchanged by SIM-1.
* **OOS-SIM1-4** — `crates/simulator/src/bin/fuzzer.rs:320-328` and
  `crates/simulator/tests/local_game.rs:74-90` build a command-zone object without
  `builder.player_commander(..)`, so `commander_ids` is empty and their games are not Commander
  games (no tax, no CR 903.9a return, no CR 903.10a commander damage). `setup.rs:258-270`
  documents exactly this failure mode. Deliberately unfixed: fixing it moves every recorded
  fuzz seed.

---

## 4. Test matrix

**New file**: `crates/simulator/tests/commander_cast.rs` (a fifth top-level target alongside
`local_game.rs`, `local_game_playthrough.rs`, `local_game_human_actions.rs`, `setup.rs`;
SR-9a's one-target rule governs `crates/engine/tests`, not this crate).

**Fixture requirement, stated once**: unlike `tests/local_game.rs::build_state`, every fixture
here **must** call `builder.player_commander(pid, cid)` in addition to placing the object in
`ZoneId::Command(pid)` — otherwise `commander_ids` is empty and every assertion below passes
or fails vacuously (§0.3). Add a helper `fn commander_state(..)` in the new file and give it a
doc comment saying so.

| # | Test | What it asserts | How it is proven to discriminate |
|---|---|---|---|
| **T1** | `test_sim1_commander_offered_at_sorcery_speed` | 0 tax, affordable, PreCombatMain + empty stack + active + priority ⇒ `legal_actions` contains `CastSpell { card: <commander obj> }` | Revert Step 5 ⇒ absent. The only test that fails on the enumeration alone. |
| **T2** | `test_sim1_commander_withheld_at_instant_speed` | same fixture, `Step::Upkeep` (or opponent's turn) ⇒ **no** `CastSpell` naming the commander | Replace `can_cast_at_this_time` with `true` ⇒ offered. Proves the timing was mirrored, not skipped. |
| **T2b** | `test_sim1_commander_offered_at_instant_speed_under_a_flash_grant` | same instant-speed fixture **plus** an active `FlashGrantFilter::AllSpells` grant for this player ⇒ offered | Delete the flash-grant branch of the mirrored predicate ⇒ withheld. Pins that the mirror is the *whole* hand-loop logic, incl. CR 601.3b. (Model the grant on an existing engine flash-grant test — grep `flash_grants` under `crates/engine/tests/`.) |
| **T3** | `test_sim1_taxed_commander_is_withheld_when_only_the_printed_cost_is_affordable` | **THE criterion-5985 test.** Commander `{1}{W}` (MV 2), `commander_tax[cid] = 1` ⇒ real cost `{3}{W}` (MV 4). Exactly **2** untapped Plains, empty pool ⇒ **not** offered | Revert the offer gate to `obj.characteristics.mana_cost` ⇒ offered, and then `process_command` **rejects** it. Assert both halves: not offered, **and** the engine really would refuse (call `process_command` directly and expect `Err`) — that turns it from a preference into the SR-38 subset property. |
| **T4** | `test_sim1_commander_offered_at_zero_tax_with_exactly_the_printed_cost` | same fixture, `commander_tax` absent/0, 2 Plains ⇒ offered | Non-vacuity partner for T3: proves T3's withholding is the tax and not the fixture. |
| **T4b** | `test_sim1_taxed_commander_offered_once_the_taxed_cost_is_affordable` | tax 1, **4** Plains ⇒ offered | Proves the helper does not simply suppress every taxed commander. |
| **T5** | `test_sim1_a_non_commander_object_in_the_command_zone_is_never_offered` | a second card object with a mana cost placed in `Command(P1)` but **absent from `commander_ids`**, fully affordable ⇒ not offered; and `process_command` on a hand-built `CastSpell` for it returns `Err` naming CR 903.8 | Delete the `commander_ids` filter ⇒ offered, and the engine's `casting.rs:767-771` rejects it. Also the **fuzzer-shape** test: `commander_ids` empty is exactly `fuzzer.rs`'s state (§0.3). |
| **T6** | `test_sim1_another_players_commander_is_never_offered` | 2 players, both with registered commanders and affordable costs. `legal_actions(state, P1)` contains no `CastSpell` naming P2's command-zone object, and vice versa | Change the scan to `ZoneId::Command(*)` / drop the per-player `commander_ids` read ⇒ cross-offer appears. |
| **T7** | `test_sim1_casting_the_commander_increments_the_tax` | drive the offer through `LocalGame::submit` with `auto_tap: true` ⇒ `Ok`; the object is no longer in `Command(P1)`; `state.player(P1).commander_tax[cid] == 1` | The offer→submit round trip. Reverting **either** Step 5 or Step 6 breaks it (no action to submit / `Rejected`). |
| **T8** | `test_sim1_human_auto_tap_pays_the_taxed_cost` | **local_game half's discriminator.** tax 1, pool **empty**, exactly enough untapped sources for the **taxed** cost ⇒ `submit(auto_tap: true)` returns `Ok` | Revert **only** Step 6 ⇒ `LocalGameError::Rejected("player does not have enough mana to pay the cost")`. Fails on Step 6 alone; T3 fails on Step 1 alone. Together they prove the two halves are independently load-bearing. |
| **T8b** | `test_sim1_a_pool_covering_only_the_printed_cost_does_not_skip_tapping` | pool already covers the **printed** cost but not the taxed cost; untapped sources exist ⇒ `Ok` | This is the exact sentence at `local_game.rs:586-588`. Pre-Step-6 the `can_pay_cost(pool, printed)` early return fires, no taps are prepended, and the cast is rejected. Retires that paragraph by execution. |
| **T9** | `test_sim1_bot_auto_tap_pays_the_taxed_cost` | a bot seat holding priority with a taxed, tap-affordable commander ⇒ after `advance()` the tax is 1 and the `PassPriority` fallback did **not** fire (assert via `command_count` / the journal, or a `Bot` stub that records what it returned) | Revert **only** Step 7 ⇒ the cast is rejected, `advance()` substitutes `PassPriority`, tax stays 0. Use a purpose-built `CastsTheCommanderBot` implementing `Bot` for determinism rather than relying on `HeuristicBot`'s scoring. |
| **T10** | `test_sim1_both_bots_choose_the_offered_commander_cast` | `HeuristicBot`: given a list containing the commander cast plus `PassPriority`, returns `Command::CastSpell { card: <commander> }` **deterministically** (it scores `50 + 10*MV` vs pass's 1). `RandomBot`: over ≥ 32 seeds it returns that command **at least once** (uniform choice) and **never** returns a malformed one | Satisfies "both bots pick it". The RandomBot half is a non-vacuity floor, not an equality — say so in the test's doc so a future reader does not tighten it into a flake. |
| **T11** | `test_sim1_commander_tax_commutes_with_flattening_and_x` | for a cost with hybrid + Phyrexian + `x_count: 2` and `tax = 3`: `flatten(apply_commander_tax(c,3), &[], &[]) == apply_commander_tax(flatten(c,&[],&[]), 3)`; `apply_commander_tax(c,3).mana_value() == c.mana_value() + 6`; hybrid/phyrexian/x_count preserved verbatim | Pure unit test against `mtg_engine::apply_commander_tax`. Re-implementing the tax locally, or flattening before taxing, breaks it. This is what licenses §2.2. |
| **T12** | `test_sim1_effective_cast_cost_is_the_identity_for_a_hand_card` | for an object in `Hand(P1)`, and for a commander card sitting in a **hand** (CR 903.9b lets one get there), `effective_cast_cost == printed` even with `commander_tax[cid] = 5` | The no-regression proof: guarantees no existing hand offer, tap plan or recorded seed can move. Drop the zone guard ⇒ fails. |
| **T13** | `test_sim1_drannith_magistrate_suppresses_the_command_zone_offer` | an **opponent** controls Drannith Magistrate ⇒ P1's commander cast is not offered; and `process_command` for it returns `Err` citing CR 101.2. Companion: the **controller's own** commander cast **is** still offered (the restriction is opponents-only) | Delete Step 4 ⇒ offered-and-rejected, a fresh SR-38 violation. The companion assertion stops the fix from over-suppressing. |
| **T14** | *(existing, no edit)* `crates/simulator/tests/local_game_playthrough.rs::test_s8_scripted_human_playthrough_is_clean_on_five_seeds` | its policy submits the cheapest zero-target castable with `auto_tap: true`, and asserts `error == None` — so a Step-5-without-Step-6 build fails it with `engine rejected a just-offered action (CastSpell)` | **A pre-existing test that becomes a SIM-1 discriminator for free.** Call this out in the commit body; it is the integration-level proof that the three sites agree. |

**Non-vacuity discipline**: every "not offered" test (T2, T3, T5, T6, T13) must also assert
that the action list is **non-empty** and contains `PassPriority`, so a fixture that silently
produced no actions at all cannot pass it.

---

## 5. Criterion 5984 — the HTTP probe, and its tension with 5987

**The tension is real and must be surfaced, not finessed.**

* `tools/play-server` is a **binary** crate (`main.rs` is the crate root; there is no
  `lib.rs`). An integration test under `tools/play-server/tests/` cannot reach `build_router`.
* Every HTTP test in this crate therefore lives in `main.rs`'s `#[cfg(test)] mod tests`
  (`:231`) and drives the router through `tower::ServiceExt::oneshot`. A machine gate
  (`test_no_socket_symbol_appears_in_the_test_region`, `:4089-4159`) *enforces* that: binding
  a real socket from an agent context is SIGKILLed.
* So criterion 5984 **cannot** be satisfied without a diff in `tools/play-server/src/main.rs`.

**Recommendation** — do not weaken either criterion; make 5987 precise and mechanically
checkable, and attest to it:

1. **Hard, unconditional**: `git diff main -- crates/engine/src crates/card-types/src` is
   **empty**. Paste the (empty) output in the task comment.
2. **Hard**: `git diff main -- crates/card-defs` is empty ⇒ 0 completeness flips, coverage
   unmoved at 1,137/1,804 = 63.0%.
3. **Scoped exception, declared up front**: `tools/play-server/src/main.rs` gains lines, and
   **every added line lies below the `#[cfg(test)] mod tests` marker**. Prove it, do not assert
   it: the crate already owns the line-anchored cutter (`test_region` / `code_only`, used by
   the gate at `:4089`). Run `git diff main -- tools/play-server/src/main.rs` and show that the
   lowest changed line number exceeds the marker's line. Nothing above the cut moves, so the
   **shipped binary is byte-identical in behaviour**.
4. Ask the coordinator to record the exception on criterion 5987 (a one-line ESM comment:
   *"5987 read as: no production diff outside `crates/simulator`; the 5984 probe is test-only
   and below `main.rs`'s `#[cfg(test)]` cut, shown by diff line numbers"*). **Do not** satisfy
   5987 silently while the diff is non-empty.

### The probe itself

Model it on the **UI-1 fixed-deck harness** (`main.rs:2893-3010`), not on the seed-swept
`COMBAT_SEED`/`TARGET_SEED` fixtures. The UI-1 pattern installs a session through
`session::new_game` with `DeckSource::Fixed` — the same constructor the handler uses, running
the same two Invariant-9 gates — and nothing about the HTTP path is stubbed. That makes the
probe deterministic with **no seed sweep**:

* **Commander**: `jadar-ghoulcaller-of-nephalia` — `{1}{B}`, Legendary Creature — Human Wizard
  1/1, **`Completeness::Complete`** (verified in `crates/card-defs/src/defs/jadar_ghoulcaller_of_nephalia.rs:89`),
  mono-black identity, **no ETB trigger** (its only ability is an end-step trigger). MV 2, so
  it is castable on the human's **second** land drop.
* **Deck**: 99 Swamps (the UI-1 rationale applies verbatim — Swamps are `Complete`, exempt from
  the singleton rule, produce exactly one mana each, so the `mana_solver` source-counting
  defect (triage F4) cannot influence what the probe observes). CR 903.5c satisfied.
* **Drive**: play a land when offered, else pass; stop when an offered action's `object_id` is
  the human's command-zone object (read out of band with a `ui1_zone`-style helper against
  `ZoneId::Command(PlayerId(1))`).
* **Assert**: that action's `kind == "CastSpell"` and `label == "Cast Jadar, Ghoulcaller of
  Nephalia"` (the label resolves because `NameIndex::from_view` already indexes
  `view.zones.command_zone` — `view.rs:781-792` — and the command zone is **not** redacted,
  CR 903.6 "face up"; **no play-server production change is needed**). Then `POST
  /api/game/action` with that index ⇒ `200`, and the object is no longer in
  `ZoneId::Command(1)`.
* **Second probe (pre-fix record)**: the same drive on a build with Step 5 reverted must find
  **no** action naming the command-zone object. Run it once during implementation to prove the
  probe discriminates, then convert the observation into the test's doc comment (the
  established pattern in this file) rather than shipping a `#[ignore]`d twin.

**Turn budget**: bound the drive with a `SIM1_MAX_STEPS` const well above the observed cost,
and panic printing the last payload — the same failure ergonomics as `drive_until`
(`main.rs:1619-1622`).

---

## 6. Verification checklist

- [ ] `cargo check -p mtg-simulator`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo fmt --check` **and** `tools/check-defs-fmt.sh` (SR-35)
- [ ] `cargo build --workspace` (SR-3 seal gate)
- [ ] `cargo test --workspace --no-fail-fast`
- [ ] **PROTOCOL / HASH executed, not predicted**: run the protocol-fingerprint gate and
      `cargo test --test core hash_schema`; both must report **33** and **70** unmoved. (The
      dispatch-adjacent docs have been stale on this number before — PB-DX6 bumped PROTOCOL to
      33 and CARDS-1 found a criterion still saying 32.)
- [ ] `git diff main -- crates/engine/src crates/card-types/src` **empty** (paste output)
- [ ] `git diff main -- crates/card-defs` **empty**; regenerate `tools/authoring-report.py` and
      confirm the report body is byte-identical (coverage 63.0% unmoved)
- [ ] `tools/play-server` diff confined below the `#[cfg(test)]` cut (§5, item 3)
- [ ] `crates/simulator/tests/local_game_playthrough.rs` green **with no edits** (T14)
- [ ] `crates/simulator/tests/local_game.rs` green with no edits (immune by §0.3 — if it moves,
      something is wrong with the `commander_ids` filter, not with the test)
- [ ] Benches spot-checked (`full_turn_4p`, `priority_cycle_4p`): expect noise only — the new
      loop runs over a 1-element zone per priority grant.

---

## 7. Risks

### R1 — Recorded fuzz seeds (LOW, provable)
`mtg-fuzzer` never registers `commander_ids` (§0.3), so the offer is structurally
unreachable there. **T5 is the assertion form of this claim** (a command-zone object with
empty `commander_ids` yields no offer). If a future session teaches the fuzzer
`player_commander`, every recorded seed moves — that is OOS-SIM1-4's whole content, and it is
deliberately not taken here.

### R2 — `crates/simulator/tests/local_game.rs` (LOW, provable)
Same shape as the fuzzer (`:74-90`, no `player_commander`). Immune. If any test in that file
moves, treat it as a **bug in the filter**, not as an expected perturbation, and stop.

### R3 — `local_game_playthrough.rs`, 5 seeds (MEDIUM — this one really does move)
This file **does** use `setup::build_initial_state`, so commanders are registered and the new
offer is live. Its policy prefers *"the cheapest castable spell that needs no announcement"*,
and a commander is exactly that, so **turn order, board state and command counts will change
on some or all of the five seeds**.

What the test actually asserts is robust to that: `error == None`, `violations.is_empty()`,
`leaked_tokens.is_empty()`, `outcome ∈ {GameOver, MaxTurns}`, `decisions > 0`, and the suite
level `PlayLand`/`DeclareAttackers` coverage set. **No turn or command count is pinned.**

Two failure modes to watch, and how to A/B-explain each:

* `error = "engine rejected a just-offered action (CastSpell)"` ⇒ **the SIM-1 fix is
  incomplete**, almost certainly Step 6. This is T14 doing its job; do not "fix" it by
  narrowing the offer.
* `all_kinds` loses `DeclareAttackers` ⇒ the games now spend their mana on commanders and never
  reach combat within `MAX_TURNS = 25`. If this happens, **do not** relax the assertion. A/B it:
  run the file on `main` and on the branch, print the per-seed `println!` line (`:476-488`)
  from both, and diff. If combat is genuinely lost, the honest fix is to raise `MAX_TURNS` (a
  test-config change with a comment naming SIM-1) — not to delete coverage.
* The `test_s8_playthrough_is_reproducible_from_the_seed_alone` test compares a seed against
  itself, so it cannot break from a shifted trajectory; if it breaks, the cause is
  nondeterminism (OOS-M11-3 / OOS-DP3-9 territory) and is not SIM-1's.

### R4 — `tools/play-server` fixtures (MEDIUM, one specific fixture)
Audited each:

* `SEED = 0` / `TARGETED_SPELL` (`drive_to_targeted_spell`, `:428-452`) — policy is *land, else
  index 0 (= `PassPriority`, always first)*. It never casts, so it is **unaffected**.
* `UI1_SEED = 184` (`ui1_drive_to_question`, `:3030-3069`) — policy is *land, else first
  `CastSpell`*, but the fixture deliberately pins a **7-mana** commander
  (`razaketh-the-foulblooded`) "unreachable inside the probe's window, so neither seat's
  commander can enter the battlefield and perturb the drive" (`:2900-2905`). The offer is gated
  on `can_afford`, so it is never made. **Unaffected — and the fixture author already
  anticipated exactly this.**
* `COMBAT_SEED = 6` with `develop: true` (`drive_until`, `:1562-1623`) — policy casts *"the
  first spell that announces nothing"*, which a commander is. **This one can move.** If
  `test_*` fixtures using `COMBAT_SEED` start panicking with *"seed 6 did not reach the fixture
  within 700 decisions"*, re-observe per the file's own documented protocol (`:1514-1527`): a
  temporary `#[ignore]`d `oneshot` sweep probe over `players ∈ {2,4} × seed ∈ 0..12`, pin the
  new constant, delete the probe, and note *"moved by SIM-1"* in the constant's doc.
* `actions[0]["kind"] == "PassPriority"` (`:492`) — safe: `PassPriority` is pushed first
  (`legal_actions.rs:394`) and SIM-1 appends.

### R5 — Bot livelock if the offer and the payment disagree (HIGH if the fix is partial)
`HeuristicBot` scores `CastSpell` at `50 + 10 * mana_value` (`heuristic_bot.rs:180-192`) —
a commander is typically the **highest-MV** thing available, so it will be chosen
preferentially — and `RepeatKey::of` returns `None` for `CastSpell` (`:52-63`), so **nothing
caps a repeated attempt**. With Step 5 but not Step 7, a bot would pick the taxed cast, have it
rejected, fall through to `advance()`'s `PassPriority` fallback (`:463-469`), and be re-offered
the identical action at its next priority — burning the game down through
`max_consecutive_passes` instead of playing. T9 exists specifically to catch this.

### R6 — TUI drift (LOW, filed)
`tools/tui/src/play/app.rs:280-294` is a fourth printed-cost auto-tap. Its **human** path reads
`app.hand_objects()` so a TUI human still cannot select a commander (no regression, no new
capability); its **bot** path will now mis-plan taxed commander casts and absorb the rejection.
Out of scope by criterion 5987; filed as **OOS-SIM1-2**.

### R7 — Emblems and CR 903.9a returns in the command zone (LOW, handled)
CR 408.1 makes the command zone a home for non-commander objects. The `commander_ids` filter,
not the zone, is what keeps the provider a subset of the engine; T5 pins it. An emblem also has
no `mana_cost`, so `effective_cast_cost` returns `None` and it is skipped twice over.

### R8 — Partner commanders (LOW, handled)
`commander_ids` is a `Vector<CardId>` and `commander_tax` is keyed per `CardId`, so two
commanders in one command zone are enumerated independently and taxed independently, exactly as
`casting.rs:2657-2663` does. Worth one assertion inside T6's fixture if cheap; not worth its own
test.

### R9 — The helper's `None` arm is a silent skip (LOW, deliberate)
`effective_cast_cost` returns `None` for a missing object / absent mana cost. At the offer gate
that means "do not offer" (correct — the engine would refuse a costless cast anyway); at the two
auto-tap sites it means "prepend no taps", and the engine then refuses the cast loudly. That is
the same discipline `auto_tap_commands_for`'s doc block already argues for its three `?`s
(`:559-570`); keep the argument, and update the count (Step 6).

---

## 8. What this batch explicitly does NOT do

* No engine change. No wire change. PROTOCOL 33 / HASH 70 unmoved.
* No card-def change; coverage unmoved at 63.0%.
* Does **not** teach the fuzzer or `tests/local_game.rs` to register commanders (OOS-SIM1-4).
* Does **not** fix the TUI (OOS-SIM1-2), the hybrid-plan channel on `CastSpell` (OOS-SIM1-1),
  the two unmirrored spell-count restrictions (OOS-SIM1-3), the pool-blind solver or the
  source-counting defect (triage F3/F4, OOS-M11-2).
* Does **not** touch cost reductions, Thalia-style increases, or CR 106.12 restricted mana —
  the surviving halves of `local_game.rs:581-591`'s limitation paragraph, which Step 6 must
  narrow rather than delete.
