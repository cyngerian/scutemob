# SR-34 — affected card definitions (empirical enumeration)

**Date**: 2026-07-17 · **Task**: `scutemob-90` · **Status**: scoping only, no source changed.

## Method

Enumerated from the **compiled registry** (`mtg_engine::all_cards()`, 1,749 defs), never from
source text — per the CLAUDE.md regex trap (`abilities:\s*vec!\[\s*\]` also matches inside
`mana_abilities: vec![]`).

A temporary probe test (`crates/engine/tests/core/sr34_probe.rs`, wired via a `mod` line in
`tests/core/main.rs`) walked every `AbilityDefinition::Activated { cost, effect, targets, .. }`
in every def and printed two tables. **The probe has been deleted and its `mod` line removed;
`git status` is clean of both.**

Predicates used:

- **Mana-producing**: the 10 variants of `rules::mana::is_mana_producing_effect` (CR 605.1b),
  applied to the leaf *and* via a `serde_json` walk of the effect tree, so nesting under
  `Sequence` / `Choose` / `Conditional` / `ForEach` is covered by construction rather than by an
  exhaustive matcher that under-reports the day a new recursion site is added (SR-33's method).
- **`try_as_tap_mana_ability`**: **replicated as a bool in the probe** rather than made
  `pub(crate)` — the task allowed either; replicating meant **zero source edits**. The replica
  mirrors `testing/replay_harness.rs:3635` exactly (AddMana-nonempty / AddManaAnyColor /
  AddManaFilterChoice / AddManaScaled / the 2-element `Sequence[AddMana, DealDamage{Controller,
  Fixed}]` pain-land arm). If that function moves, this replica is stale — it was read at the
  commit of this task.

---

## Table 1 — mana-producing activated abilities whose cost is **not** bare `Cost::Tap`

These are the SR-34 core: `enrich_spec_from_def` (replay_harness.rs:2115) gates mana-ability
lowering on `matches!(cost, Cost::Tap)`, so each of these registers as a **stack-using activated
ability** instead of a mana ability — a CR 605.1a/605.3b violation (an ability is a mana ability
because of what it *does*, not what it costs).

**39 rows, 39 distinct cards** (one affected ability per card).

### Counts matrix — cost shape × completeness

| Cost shape | Complete | Partial | KnownWrong | Inert | **Total** |
|---|---:|---:|---:|---:|---:|
| `Sequence[Mana + Tap]` | 14 | 1 | 7 | 0 | **22** |
| `Sequence[Tap + PayLife]` | 3 | 1 | 3 | 0 | **7** |
| `RemoveCounter` (no tap) | 2 | 1 | 0 | 0 | **3** |
| `Sacrifice(filter)` (no tap) | 2 | 0 | 0 | 0 | **2** |
| `Mana` (only) | 0 | 1 | 1 | 0 | **2** |
| `Sequence[Tap + SacrificeSelf]` | 1 | 0 | 0 | 0 | **1** |
| `Sequence[Tap + Sacrifice(filter)]` | 1 | 0 | 0 | 0 | **1** |
| `SacrificeSelf` (no tap) | 0 | 0 | 1 | 0 | **1** |
| **Total** | **23** | **4** | **12** | **0** | **39** |

### Full list of affected `Complete` defs (23)

These are the ones criterion 4767 must reconcile — each either works end-to-end after the fix or
gets a truthful marker.

`Sequence[Mana + Tap]` — 14:

| Card | Cost | Effect |
|---|---|---|
| Boros Signet | `{1},{T}` | `AddMana {W}{R}` |
| Dimir Signet | `{1},{T}` | `AddMana {U}{B}` |
| Golgari Signet | `{1},{T}` | `AddMana {B}{G}` |
| Izzet Signet | `{1},{T}` | `AddMana {U}{R}` |
| Orzhov Signet | `{1},{T}` | `AddMana {W}{B}` |
| Rakdos Signet | `{1},{T}` | `AddMana {B}{R}` |
| Simic Signet | `{1},{T}` | `AddMana {U}{G}` |
| Darkwater Catacombs | `{1},{T}` | `AddMana {U}{B}` |
| Viridescent Bog | `{1},{T}` | `AddMana {B}{G}` |
| Magnifying Glass | `{1},{T}` | `AddMana {C}` |
| Cabal Coffers | `{2},{T}` | `AddManaScaled {B} × Swamps` |
| Cabal Stronghold | `{3},{T}` | `AddManaScaled {B} × basic Swamps` |
| Crypt of Agadeem | `{2},{T}` | `AddManaScaled {B} × creature cards in GY` |
| Three Tree City | `{2},{T}` | `AddManaOfAnyColorAmount × chosen-type creatures` |

`Sequence[Tap + PayLife]` — 3:

| Card | Cost | Effect |
|---|---|---|
| Mana Confluence | `{T}, Pay 1 life` | `AddManaAnyColor` |
| Staff of Compleation | `{T}, Pay 2 life` | `AddManaAnyColor` |
| Voldaren Estate | `{T}, Pay 1 life` | `AddManaAnyColorRestricted(Vampire)` |

`RemoveCounter` (no tap) — 2:

| Card | Cost | Effect |
|---|---|---|
| Druids' Repository | `Remove a charge counter` | `AddManaAnyColor` |
| Gemstone Array | `Remove a charge counter` | `AddManaAnyColor` |

`Sacrifice(filter)` (no tap) — 2:

| Card | Cost | Effect |
|---|---|---|
| Ashnod's Altar | `Sacrifice a creature` | `AddMana {C}{C}` |
| Phyrexian Altar | `Sacrifice a creature` | `AddManaAnyColor` |

`Sequence[Tap + SacrificeSelf]` — 1:

| Card | Cost | Effect |
|---|---|---|
| Goldhound | `{T}, Sacrifice this` | `AddManaAnyColor` |

`Sequence[Tap + Sacrifice(filter)]` — 1:

| Card | Cost | Effect |
|---|---|---|
| Phyrexian Tower | `{T}, Sacrifice a creature` | `AddMana {B}{B}` |

### Non-`Complete` affected defs (16) — representative list

`Partial` (4): **Abstergo Entertainment** (`{1},{T}` → `AddManaAnyColor`), **Gnarlroot Trapper**
(`{T}, Pay 1 life` → `AddManaRestricted {G}`), **Ramos, Dragon Engine** (`Remove five +1/+1
counters` → `AddMana WWUUBBRRGG`), **Simian Spirit Guide** (`Mana(∅)` → `AddMana {R}`).

`KnownWrong` (12): the seven **filter lands** (Cascade Bluffs, Fetid Heath, Flooded Grove, Graven
Cairns, Rugged Prairie, Sunken Ruins, Twilight Mire — all `Sequence[Mana(hybrid) + Tap]` →
`AddManaFilterChoice`), the three **horizon lands** (Fiery Islet, Nurturing Peatland, Silent
Clearing — `Sequence[Tap + PayLife(1)]` → `AddManaChoice`), **Elvish Spirit Guide**
(`Mana(∅)`), **Food Chain** (`SacrificeSelf`).

`Inert`: **0**.

---

## Table 2 — bare `Cost::Tap` + mana-producing, but `try_as_tap_mana_ability` returns `None`

The cost widening alone will **not** fix these: the cost is already bare `Tap`, but the *effect*
shape is unrecognised, so they register nothing today and would still register nothing.

**11 rows.**

| Card | Marker | Effect | Why dropped |
|---|---|---|---|
| Deathrite Shaman | Complete | `Sequence[ExileObject{DeclaredTarget 0}, AddManaAnyColor]` | **False positive** — see below |
| Maelstrom of the Spirit Dragon | Complete | `AddManaAnyColorRestricted(Dragon/Omen)` | variant unhandled |
| Secluded Courtyard | Complete | `AddManaAnyColorRestricted(ChosenTypeCreaturesOnly)` | variant unhandled |
| Unclaimed Territory | Complete | `AddManaAnyColorRestricted(ChosenTypeCreaturesOnly)` | variant unhandled |
| Temple of the Dragon Queen | Complete | `AddManaOfChosenColor { amount: 1 }` | variant unhandled |
| Cavern of Souls | Partial | `AddManaAnyColorRestricted(ChosenTypeCreaturesOnly)` | variant unhandled |
| Haven of the Spirit Dragon | Partial | `AddManaAnyColorRestricted(Dragon)` | variant unhandled |
| The Seedcore | Partial | `AddManaAnyColorRestricted(Phyrexian)` | variant unhandled |
| The Great Henge | Partial | `Sequence[AddMana {G}{G}, GainLife 2]` | `Sequence` arm only matches `[AddMana, DealDamage]` |
| Strixhaven Stadium | Partial | `Sequence[AddMana {C}, AddCounter]` | same |
| Glistening Sphere | KnownWrong | `AddManaChoice { count: Fixed(3) }` | variant unhandled |

Marker split: **Complete 5** (4 real + 1 false positive), **Partial 5**, **KnownWrong 1**, **Inert 0**.

**Deathrite Shaman is a false positive.** Its `AbilityDefinition` carries
`targets: [TargetCardInGraveyard(...)]`, and its oracle text really does read *"Exile **target**
land card from a graveyard. Add one mana of any color."* Per CR 605.1a a targeting ability is not
a mana ability, so a stack-using activated ability is **correct**. Table 2's filter did not check
`targets`; Table 1's did (and printed `targets=0` on every row, confirming no Table-1 row targets).
**Real Table 2 count: 10.**

---

## What a `ManaAbility { mana_cost, life_cost }` extension fixes vs. what it doesn't

The proposed extension adds an activation-cost payload to `ManaAbility` (which today carries only
`produces` / `requires_tap` / `sacrifice_self` / `any_color` / `damage_to_controller`).

### Fixed by `mana_cost` + `life_cost` alone — **26 of 39** (17 Complete)

| Group | n | Complete | Note |
|---|---:|---:|---|
| `Sequence[Mana(generic) + Tap]` → `AddMana` / `AddManaScaled` / `AddManaOfAnyColorAmount` | 15 | 14 | Signets, Darkwater Catacombs, Viridescent Bog, Magnifying Glass, Cabal Coffers/Stronghold, Crypt of Agadeem, Three Tree City, Abstergo Entertainment. `mana_cost` alone. |
| `Sequence[Tap + PayLife]` → `AddManaAnyColor` / `AddManaRestricted` | 4 | 2 | Mana Confluence, Staff of Compleation, Voldaren Estate, Gnarlroot Trapper. `life_cost` alone. |
| `Sequence[Tap + SacrificeSelf]` | 1 | 1 | Goldhound — `sacrifice_self` **already exists** on `ManaAbility`; only the `matches!(cost, Cost::Tap)` gate blocks it. Cheapest win in the set. |
| `Sequence[Mana(hybrid) + Tap]` → `AddManaFilterChoice` | 7 | 0 | Filter lands. `try_as_tap_mana_ability` already handles `AddManaFilterChoice`; needs **hybrid** `mana_cost`, see below. |

### Needs more than the two new fields — **13 of 39** (6 Complete)

| Shape | Cards | What's missing |
|---|---|---|
| **Hybrid mana cost** | 7 filter lands (all `KnownWrong`) | `mana_cost` must express `{U/R}` etc. `ManaCost.hybrid: Vec<HybridComponent>` exists on the DSL side, but paying a hybrid cost is a *choice* — and per SR-33's decision, choice for stackless mana abilities is channelled through `TapForMana{ability_index}`. Same shape as SR-33's dual-land fix: **one ability per hybrid half**. These are already `KnownWrong` and are not on criterion 4767's hook. |
| **`Cost::RemoveCounter`** | Druids' Repository (C), Gemstone Array (C), Ramos (P) | Needs a `counter_cost` field + a `handle_tap_for_mana` payment path. `requires_tap: false` — these have **no tap at all**; verify the no-tap path works. |
| **`Cost::Sacrifice(filter)`** (sacrifice *another* permanent) | Ashnod's Altar (C), Phyrexian Altar (C), Phyrexian Tower (C) | `sacrifice_self` is not this. Needs a `sacrifice_other: Option<SacrificeFilter>` **plus a way to name which permanent** — `TapForMana` has no payload for it. This is the one group that pushes the Command shape. Ashnod's/Phyrexian Altar also have `requires_tap: false`. |
| **`AddManaChoice`** | 3 horizon lands (KW), Glistening Sphere (KW, Table 2) | `AddManaChoice` is an SR-33-gated stub (adds one **colorless**, ignores `count`). SR-34 un-demotes the horizon lands **only if** they are rewritten to per-colour abilities (tainted_field pattern), not by widening the cost. `Cost::Sequence[Tap + PayLife(1)]` + one ability per colour. |
| **`Cost::Mana(∅)` mismodelling** | Elvish Spirit Guide (KW), Simian Spirit Guide (P) | Not a cost-widening problem at all — the real cost is "exile this card from your hand", which needs `ActivationZone::Hand` + an exile-self-from-hand cost. **Out of SR-34 scope.** |
| **`Cost::SacrificeSelf` mismodelling** | Food Chain (KW) | Def is a known-wrong placeholder (sacrifices Food Chain itself). Out of scope. |

### Table 2 shapes (effect-side, orthogonal to the cost widening)

| Shape | n | Fix |
|---|---:|---|
| `AddManaAnyColorRestricted` | 6 (3 Complete) | Add an arm to `try_as_tap_mana_ability`. `ManaAbility` has no field for a spend restriction — needs one, or `any_color: true` ships a lie (unrestricted mana). |
| `AddManaOfChosenColor` | 1 (Complete) | Add an arm; reads the source's chosen colour. |
| `Sequence[AddMana, <non-damage rider>]` | 2 (both Partial) | Generalise the pain-land `Sequence` arm beyond `[AddMana, DealDamage{Controller, Fixed}]`. Both defs are `Partial` for unrelated reasons. |
| `AddManaChoice` | 1 (KnownWrong) | SR-33 stub; gated out of Complete already. |

---

## Surprising things

1. **`Cost::Tap` is not the only thing `enrich_spec_from_def` mis-gates — 4 of 39 affected
   abilities have no tap in their cost at all** (Ashnod's Altar, Phyrexian Altar, Druids'
   Repository, Gemstone Array; plus Food Chain). `ManaAbility::requires_tap` exists and defaults
   are all `true` in `try_as_tap_mana_ability` — every one of its five return sites hardcodes
   `requires_tap: true`. Nothing in the codebase has ever produced a `requires_tap: false` mana
   ability from a def. **The `false` path is unexercised** and should be treated as unproven, not
   as existing capability (the `feedback_verify_full_chain` trap: variant existence ≠ working
   chain).

2. **Goldhound is a one-line fix.** `sacrifice_self` already exists on `ManaAbility`, and
   `Sequence[Tap + SacrificeSelf]` is exactly the Treasure-token shape the field was built for.
   The *only* thing stopping it is the `matches!(cost, Cost::Tap)` gate. That means the gate has
   been suppressing a capability the engine already has — which is the same class of finding as
   SR-33's `card_registry_gate` root cause.

3. **The Signet cycle is 7 of the 23 affected Complete defs.** These are among the most-played
   Commander cards in existence and all seven are `Complete` while using the stack for mana. A
   `{1},{T}` Signet on the stack cannot be activated in response to a counterspell to pay for a
   Force of Will — the observable failure is a full mana-availability bug, not a cosmetic one.

4. **`ManaAbility::any_color` is documented as "defaults to colorless until interactive color
   choice is implemented."** That is a live simplification affecting *already-registered* mana
   abilities and is **not** SR-34 — but it means fixing Mana Confluence's cost lowering yields an
   ability that produces colorless. **Mana Confluence, Staff of Compleation, Phyrexian Altar,
   Goldhound and Druids'/Gemstone would go from "wrong (uses the stack)" to "wrong (only makes
   {C})".** Any criterion-4767 reconciliation that counts these as "works end-to-end" on the
   strength of the cost fix alone is repeating SR-33's `megrim.rs` calibration error. Either the
   `any_color` choice channel lands too (`TapForMana{ability_index}` per-colour, per
   `memory/decisions.md` 2026-07-17), or these get truthful markers.

5. **Zero `Inert` defs in either table**, which is expected — an inert def registers no behaviour
   and therefore has no activated ability to mis-gate.

6. **The three horizon lands SR-33 demoted are in Table 1, not Table 2.** SR-34's charter says it
   "un-demotes the 3 horizon lands" — but they are `Sequence[Tap + PayLife(1)] → AddManaChoice`,
   i.e. **both** an SR-34 cost problem **and** an SR-33 stub problem. The cost widening is
   necessary and not sufficient; they need the per-colour rewrite as well.

7. **Deathrite Shaman looks like a bug and isn't.** Worth writing down because the obvious
   "widen `try_as_tap_mana_ability`" instinct would try to lower it, and CR 605.1a forbids that.
   The `targets.is_empty()` check is load-bearing and `enrich_spec_from_def`'s mana-lowering loop
   **does not currently check it** — it destructures `AbilityDefinition::Activated { cost, effect,
   .. }` and drops `targets` on the floor. Today that is harmless only because `Cost::Tap` +
   targets + a recognised mana effect happens not to co-occur in any def except Deathrite, whose
   `Sequence[ExileObject, AddManaAnyColor]` shape `try_as_tap_mana_ability` rejects for an
   unrelated reason. **Widening either the cost gate or the effect matcher without adding
   `targets.is_empty()` will silently make Deathrite Shaman a mana ability.** That is a
   CR 605.1a violation the current code avoids by luck.
