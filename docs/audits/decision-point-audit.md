# Decision-Point Surface Audit

<!-- last_updated: 2026-07-26 -->

**Date**: 2026-07-26
**Task**: `scutemob-148`
**Bug class**: A point where the Comprehensive Rules give a *player* a choice, but the engine
makes that choice itself — deterministically, legally, and invisibly — because no `Command`
carries the player's answer.
**Scope**: read-only audit of the whole decision surface. No engine, simulator, card-def or
test change. The only files this task writes are this document and its index line in
[README.md](README.md).
**Motivation**: the 2026-07-26 finding that triggered-ability targets are auto-selected
engine-side (`abilities.rs:7174-7500`, proposed seed OOS-M11-4; renumbered from OOS-M11-3 at collection — scutemob-147 independently claimed OOS-M11-3 for fuzzer long-game nondeterminism), plus M11-local building a
human-input game loop whose decision surface must be known to be complete before the loop is
designed around it.

> **Severity-tally disambiguation** — same convention as
> [layer-bypass-audit.md](layer-bypass-audit.md). The class-B and class-D findings catalogued
> here are this audit's own class. They are **not** the "0 HIGH / 2 MEDIUM / 6 LOW" engine
> tally in `CLAUDE.md` → Current State, which counts the milestone-review issue index. Both
> are correct at once. Nothing here is scheduled yet; §8 proposes a queue, it does not create
> one.

---

## 1. The invariant

Per [methodology.md](methodology.md) Step 1, the property this audit tests is stated
declaratively:

> **DP-INV**: For every decision the CR assigns to a player, the engine either (a) obtains
> that player's answer through a `Command`, or (b) refuses to admit into a game any card
> that would reach the decision.

Branch (b) already exists and is machine-enforced. Architecture Invariant 9 / SR-2 make
`validate_deck` reject any card whose `CardDefinition` is not `Complete`, and the reason
given in the source is exactly the one that matters here:

```rust
    /// The definition exists but is marked `Inert`, `Partial`, or `KnownWrong`
    /// (see `cards::Completeness`). Such a card would take actions the rules
    /// engine never emits, or emit actions the card does not have, producing a
    /// state history that cannot be correctly rewound. Surfaced here, at deck
    /// build, rather than silently misbehaving on turn 6.
```
— `crates/engine/src/rules/commander.rs:46-50`, enforced at `commander.rs:232`.

An engine that picks a player's target, keeps or bottoms their scried cards, or discards
their cleanup card produces precisely "a state history that cannot be correctly rewound",
because the history records a decision no player made. DP-INV is therefore the same
invariant Architecture Invariant 9 already asserts, applied to *choices* rather than to
*abilities*.

**The gate is narrower than the invariant.** `crates/engine/tests/core/effect_choose_gate.rs`
(SR-33) bars exactly three DSL variants from `Complete`:

| gate test | variant barred |
|---|---|
| `no_complete_def_uses_the_choose_stub` (`:84`) | `Effect::Choose` |
| `no_complete_def_uses_the_may_pay_or_else_stub` (`:109`) | `Effect::MayPayOrElse` |
| `no_complete_def_uses_the_add_mana_choice_stub` (`:149`) | `Effect::AddManaChoice` |

Every other auto-chosen decision ships `Complete`. That is the hole this audit measures.

---

## 2. Classification and method

### 2.1 The taxonomy (from the task brief)

| class | meaning |
|---|---|
| **A** | Player-hooked: the choice rides a `Command` field and the handler reads it and acts on it. |
| **B** | Deterministic auto-fallback: the engine picks legally, without player input. Legal game state, zero agency. |
| **C** | Gated/stubbed: the decision is unimplemented and the corpus gate keeps affected cards out of `Complete`. |
| **D** | Missing or wrong vs CR: the behaviour diverges from the rules, or a hook exists that is never honoured. |

### 2.2 Evidence standard

A site is class **A** only if the choice was traced from a `Command` field, through
`process_command`'s dispatcher, into the handler line that consumes it. "The `Command` has a
field" was never accepted as evidence — three findings below (`face_down_kind`,
`ChooseDungeonRoom`, `Command::OrderReplacements` in reply to a `WouldDraw` prompt) are
exactly cases where a field or command exists and is discarded.

Every CR number cited here was looked up against `.scryfall-cache/MagicCompRules.txt` via the
mtg-rules MCP. Two corrections to the task brief resulted:

- **Scry is CR 701.22, not 701.19.** CR 701.19 is *Regenerate*. Surveil is **CR 701.25**.
- **CR 602.2e does not exist.** `abilities.rs:1384` cites it; CR 602.2 has only 602.2a/b.
  See finding **DP-1**.

Line numbers were verified by reading the files, and the four delegated research legs were
spot-checked against source before their claims were adopted. One agent-reported count was
wrong and is corrected here: `Effect::MayPayThenEffect` appears in **11** effectively-`Complete`
defs, not 12 — `mana_vault`, `temur_sabertooth`, `call_of_the_ring`, `boggart_shenanigans`,
`roil_elemental`, `ezuri_stalker_of_spheres`, `ruthless_technomancer` and `vampire_gourmand`
are `partial` or `inert`.

### 2.3 Corpus-impact method

Counts below come from a regex sweep over `crates/card-defs/src/defs/*.rs`, **excluding
`defs/mod.rs`** (the module file, which is caught by the glob but is not a card) — 1,804 def
files. A def is "effectively `Complete`" if it says `Completeness::Complete` **or** carries no
`completeness:` line at all (the DSL default) — **1,139 files, exactly matching
`tools/authoring-report.py`.**

This is an *approximation*: it is a source regex, not the serde walk `effect_choose_gate.rs`
uses, so it will miss a variant reached only through a deeply nested generated ability and
will over-count a variant that appears only in a comment. (`Effect::Choose` is the live example
of both traps at once: the bare string appears in 119 files, **all** of them prose in doc
comments; actual code use — `Effect::Choose {` — is zero.) Word boundaries matter too:
`Discover\b` is the mechanic (CR 701.57) and deliberately does not match the card name
*Kindred Discovery*, which is unrelated and `inert` besides.

**Method, so §10's re-derivation instruction is actionable.** Each §3.1 row is a regex over the
file's source text; a def counts once per row it matches. Nineteen rows are a plain match on the
variant name as written in the table. Two are compound predicates on the same variant and must
both be included in the union:

- *targeted triggered ability* = `AbilityDefinition::Triggered` **and** `targets:\s*vec!\[\s*TargetRequirement::`
- *modal triggered ability* = `AbilityDefinition::Triggered` **and** `modes:\s*Some`

Runnable script: `scratchpad/count-classb.py` from this task's session
(`/tmp/claude-1000/-home-skydude-projects-scutemob--worktrees-scutemob-148/…/scratchpad/`).

---

## 3. Corpus impact

### 3.1 `Complete` defs containing at least one auto-chosen decision

| decision site | defs | effectively `Complete` |
|---|---:|---:|
| targeted triggered ability (CR 603.3d) | 104 | **84** |
| `SearchLibrary` (CR 701.23) | 108 | **74** |
| `Proliferate` (CR 701.34a) | 35 | 25 |
| `DiscardCards` / `WheelHand` / `WheelDraw` (CR 701.8) | 32 | 23 |
| `Effect::Scry` (CR 701.22a) | 20 | 16 |
| `SacrificePermanents` (CR 701.21a) | 21 | 11 |
| `MayPayThenEffect` (CR 118.12) | 19 | 11 |
| `ChooseColor` / `ChooseCreatureType` | 22 | 10 |
| `LookAtTopThenPlace` / `RevealAndRoute` | 24 | 10 |
| `Effect::Surveil` (CR 701.25a) | 9 | 8 |
| `CounterUnlessPays` (CR 118.12a) | 7 | 7 |
| modal triggered ability (CR 603.3c) | 8 | 5 |
| `ChangeTargets` (CR 115.7d) | 5 | 3 |
| `PutOnLibrary` (CR 701.20) | 6 | 3 |
| `Bolster` / `Amass` | 5 | 3 |
| `Connive` (CR 701.50) | 3 | 2 |
| `Discover` (CR 701.57) | 1 | 1 |
| `MayPayOrElse` (CR 118.12a) | 12 | **0** ← gated |
| `AddManaFilterChoice` | 7 | **0** ← `known_wrong` |
| `Effect::Choose` | 0 | **0** ← gated, unused |
| `TheRingTemptsYou` (CR 701.54) | 2 | **0** |

> **277 of 1,139 effectively-`Complete` defs — 24.3% — contain at least one decision the
> engine makes for the player.**

The four zero-`Complete` rows at the bottom are the control group, and they get there two
different ways. `MayPayOrElse` and `Effect::Choose` are held out by the SR-33 gate, which works
exactly as designed for the variants it names. `AddManaFilterChoice`'s seven filter lands are
held out by hand, marked `known_wrong` in PB-RS2. `TheRingTemptsYou` is simply not used by any
`Complete` def yet. The gate does not name the other seventeen rows at all.

### 3.2 Decisions that need no card at all

Six findings are in core turn structure and combat and are reachable in **every** game
regardless of decklist: priority after casting (**DP-1**), the mulligan (**DP-2**), cleanup
discard (**DP-3**), combat damage assignment (**DP-13**), same-controller trigger ordering
(**DP-14**), and the mana-pool spend order (**DP-26**).

---

## 4. Findings by decision class

Each row cites the site verified by reading. `crates/engine/src/` is elided from paths.

### 4.1 Spell casting (CR 601.2)

| choice | CR | class | site |
|---|---|---|---|
| Targets announced at cast | 601.2c | **A** | `rules/casting.rs:5805-5936` `validate_targets_inner`; two-pass slot fit + `enforce_inter_target_distinctness` (`:6119`); stored `:4522` |
| Mode announcement | 601.2b / 700.2a | **A** when supplied | `rules/casting.rs:3506-3554`; range, duplicate, `min_modes`, `max_modes` all checked |
| Mode announcement, **omitted** | 601.2b / 700.2a | **A** *(was **D**)* | **A since PB-DP3** (`scutemob-151`) — an omitted mode announcement is rejected before costs are determined or paid (`rules/casting.rs:3507-3620`). One CR-702.120a-scoped exemption survives: escalate with `count > 0` announces the mode *count* via `AdditionalCost::EscalateModes` and derives the identities `0..=count` (**OOS-DP3-1**); the derived count is itself bounds-checked. Free-cast producers that bypass `handle_cast_spell` are unaffected — **DP-20** / **OOS-DP3-3**. See **DP-4** |
| Value of X | 601.2b / 107.3 | **A** | `rules/casting.rs:3975-3985`; stored `:4588` |
| Alternative cost | 118.9 | **A** | `rules/casting.rs:148-177`, kind cross-checked `:840`. Caveat at `:405-406`: flashback/escape/madness are inferred from zone+keyword and cannot be declined — **B** |
| Kicker / multikicker | 702.33d | **A** | `rules/casting.rs:2670-2705`; single-kicker >1 rejected `:2675` |
| Prototype | 702.160 | **A** | `rules/casting.rs:2336-2412`, `:4561`, `:4636-4654` |
| Additional costs (sacrifice, discard, …) | 601.2b / 118.8 | **A** for the first | `rules/casting.rs:3310-3315` errors when a required cost is omitted |
| Additional costs, 2nd and later | 118.8 | **D** | `rules/casting.rs:3307-3309` — only `required_costs[0]` is validated, despite the comment claiming otherwise |
| Convoke / Improvise / Delve membership | 702.51 / 702.126 / 702.66 | **A** | `rules/casting.rs:5429-5542`, `:5561-5650`, `:5666-5726`. Empty vec = no reduction; no auto-tap |
| Which pip a convoke creature pays | 702.51a | **B** | `rules/casting.rs:5493-5522` — colored-first, first match in WUBRG order, generic only as fallback |
| Hybrid pip payment, **supplied** | 107.4e | **A** | `CastSpellData::hybrid_choices` (`rules/command.rs:683`) → `rules/casting.rs:3991` `flatten_hybrid_phyrexian(cost, &hybrid_choices, &phyrexian_life_payments)?`. Same hook on `ActivateAbility` (`command.rs:137`) via `abilities.rs:775-782` and on `TapForMana` (`command.rs:58`) via `rules/mana.rs:252-261`. Shipped by PB-RS2 for OOS-RS-2 |
| Hybrid pip payment, **omitted** | 107.4e | **B** by design | `crates/card-types/src/state/game_object.rs:238-284` — an unindexed `{A/B}` defaults to first colour `A`; `{2/C}` to the coloured half. A short vector is the deliberate contract; an over-long one is rejected loudly (`:198-206` hybrid, `:207-215` Phyrexian, with the rationale at `:187-197`) |
| Phyrexian pip payment, **supplied** | 107.4f | **A** | `CastSpellData::phyrexian_life_payments` (`rules/command.rs:689`), same three call sites as above |
| Phyrexian pip payment, **omitted** | 107.4f | **B** by design | `game_object.rs:287-309` — `unwrap_or(false)` ⇒ pay with mana, not life. Phyrexian-**hybrid** `{A/B/P}` paid with mana always takes colour `A`; no choice channel exists for that sub-case (`:297-304`) |
| Which pool mana pays a generic pip | 601.2h | **B** | `crates/card-types/src/state/player.rs:203-244` — coloured pips first, restricted before unrestricted, then generic in fixed order C→G→R→B→U→W |
| `face_down_kind` | 702.37c | **D** | `rules/casting.rs:67` binds `_face_down_kind` and never reads it; the kind is re-derived from the def at `:4661-4683` (Disguise > Megamorph > Morph) |
| Target **division** ("divided as you choose") | **601.2d** | **D** | Nothing anywhere: no `CastSpellData` field, no `Effect` variant, no resolution path. Sole in-corpus card `crates/card-defs/src/defs/fire_covenant.rs:25-37` is `Effect::Nothing`, marked `partial` |

The engine itself never taps a land. `grep -rn "solve_mana\|auto_tap" crates/engine/src/`
is empty; the only auto-tapper is `crates/simulator/src/driver.rs:202-209`, and it emits real
`Command::TapForMana` values through the normal surface. **Architecture Invariant 3 holds
here.**

### 4.2 Activated and loyalty abilities (CR 602.2b, 606)

| choice | CR | class | site |
|---|---|---|---|
| Targets | 602.2b / 601.2c | **A** | `rules/abilities.rs:459-481`, validated *before* costs are spent (`:432-433`) |
| X in an activation cost | 107.3k | **A** | `rules/abilities.rs:713-719`; `None` ⇒ 0 |
| Modes | 700.2a / 602.2b | **A** *(was **B**)* | **A since PB-DP3** (`scutemob-151`) — `rules/abilities.rs:337-398`. The `min_modes` bypass DP-4 named here was fixed in the same PB as its cast-path twin: an omitted mode announcement on a modal activated ability is now rejected before costs are spent. `min_modes: 0` correctly accepts an empty announcement and resolves **no** mode (the Spell path hard-rejects that shape instead — deliberate asymmetry, **OOS-DP3-2**) |
| Discard-as-cost card | 602.2 | **A** | `rules/abilities.rs:849-873`; hard error when required and absent |
| Sacrifice-as-cost permanent | 602.2 / 701.21a | **A** | `rules/abilities.rs:1005-1085`; filter, "another" (CR 109.1), and can't-be-sacrificed all checked |
| **Forage**: exile 3 from graveyard **or** sacrifice a Food | **701.61a** | **B** | `rules/abilities.rs:1162-1214` — Food branch wins whenever any Food exists; Food and graveyard cards both picked lowest-`ObjectId`-first |
| Loyalty ability targets | 606 / 601.2c | **A** | `rules/engine.rs:2533-2545`, validated before loyalty is spent |
| Loyalty −X value | 606.6 | **A** | `rules/engine.rs:2552-2566` |
| Loyalty once-per-turn | 606.3 | **A** | `rules/engine.rs:2491-2495`, set `:2576` |
| Loyalty **timing** | **606.3** | **D** | `rules/engine.rs:2451-2660` contains no `priority_holder`, no `active_player`, and no split-second check — see **DP-21** |
| Modal loyalty ability | 700.2a | **D** (narrow) | `rules/command.rs:615-626` — `ActivateLoyaltyAbility` has no `modes_chosen` field; unrepresentable |
| Forecast targets | 602.2b | **D** | `rules/abilities.rs:1725-1748` wraps `targets` into `SpellTarget` with no `validate_targets_*` call at all — no requirement, count, hexproof, shroud or protection check |
| Bloodrush target | 207.2c | **D** (narrow) | `rules/abilities.rs:1845-1880` — hand-rolled checks; protection/hexproof/shroud not consulted |

### 4.3 Mana and special actions

| choice | CR | class | site |
|---|---|---|---|
| Colour of "add one mana of any colour" at a mana ability | 106.1b / 605.3b | **A** | `Command::TapForMana { chosen_color }` (`rules/command.rs:38-66`), required when `any_color`, rejected when not |
| Same colour choice inside `execute_effect` | 106.1b | **C** | `effects/mod.rs:2272-2286` adds `{C}` and ignores `count`; gated by `effect_choose_gate.rs:149/195` |
| Filter-land 3-way mana choice | 605.1a | **B** | `effects/mod.rs:2289-2299` — always one of each colour; AA and BB unreachable. All 7 filter lands are `known_wrong` (PB-RS2) |
| `AddManaOfAnyColorAmount` / `AddManaMatchingType` / `AddManaOfChosenColor` | 106.12a / 614.12 | **B** | `effects/mod.rs:2388`, `:2410`, `:2430` — colourless fallback |
| `PlayLand` | 305.1 | **A** | `rules/lands.rs:25`, priority checked `:31` |
| Foretell / Plot / Suspend / Cycling | 702.143 / 702.170 / 702.62 / 702.29 | **A** | `rules/foretell.rs:41-90`, `rules/plot.rs:48-122`, `rules/suspend.rs:52-146`, `rules/abilities.rs:1417-1472` — each checks priority and pays through `pay_cost` |
| `CrewVehicle` creature set | 702.122a | **A** | `rules/abilities.rs:8676-8761` — duplicate, self-crew, battlefield, controller, untapped, is-creature, power threshold |
| `SaddleMount` creature set | 702.171a | **A** | `rules/abilities.rs:8819+`, plus a proper sorcery-speed gate at `:8846-8864` |
| `ActivateNinjutsu` returned attacker | 702.49a | **A** | `rules/abilities.rs:2145-2259` — every clause checked |
| `ActivateCraft` material set | 702.167a | **A** for selection, **D** for legality | `rules/engine.rs:1343-1393`; but `:1357-1366` accepts `ZoneId::Graveyard(_)` for **any** player and never checks `mat_obj.controller` — you can craft with an opponent's permanents |
| `TurnFaceUp` method | 702.37e | **A** | `rules/engine.rs:1469-1596` |
| `BringCompanion` | 702.139a | **A** | `rules/commander.rs:914+`; sorcery-speed + once-per-game enforced |
| `ChooseDungeonRoom` | **309.5a** | **C**/**D** | `rules/engine.rs:526-530` — `room` bound to `_`; the command validates, returns `Ok`, emits nothing, mutates nothing. Real pick at `rules/engine.rs:2215-2217` = `exits.first()`; first dungeon hardcoded `LostMineOfPhandelver` (`:2174-2178`) |
| Ring-bearer | 701.54a | **B** | `Command::TheRingTemptsYou` has no creature field; `rules/engine.rs:2293-2297` picks the lowest-`ObjectId` creature |

### 4.4 Mulligan and commander zone

| choice | CR | class | site |
|---|---|---|---|
| Take-or-keep | 103.5 | **A** | `Command::TakeMulligan` / `KeepHand`, dispatched `rules/engine.rs:245-257` |
| The **shuffle** inside a mulligan | **103.5** | **D** — **SHIPPED (PB-DP2)** | `rules/commander.rs:826-846` — see **DP-2** |
| `cards_to_bottom` **placement** | **103.5** | **D** — **SHIPPED (PB-DP2)** | `rules/commander.rs:911-915` — see **DP-2** |
| `cards_to_bottom` count | 103.5c | **A** | `rules/commander.rs:873-885` — `len() == mulligan_count.saturating_sub(1)` |
| `cards_to_bottom` relative order | 103.5 | **A** | iterated in vector order |
| Mulligan timing | 103.5 | **D** (minor) | dispatch has only `validate_player_exists`; a mulligan is legal on turn 5 |
| Commander graveyard/exile return | 903.9a | **A**, non-blocking | `rules/commander.rs:410-420` emits `CommanderZoneReturnChoiceRequired` and pushes `pending_commander_zone_choices`; both answer commands validate and are honoured. But the pending vector is consulted only by the re-emit suppressor, `state/hash.rs:7704`, and the two handler `retain`s — never by priority, SBA or turn progression |
| Commander zone-change model | 903.9a | **D** (documented) | Modelled as an SBA *after* the move, not a replacement. `state/builder.rs:1181-1184` states the choice; the commander genuinely enters the graveyard as a new object (CR 400.7) and `CreatureDied` fires first |
| Commander hand/library redirect | **903.9b** | **B** | `state/builder.rs:1199-1229` registers an unconditional `RedirectToZone(Command)`. One applicable effect ⇒ `AutoApply` ⇒ always redirects; the owner's "may" is dropped |
| Casting from the command zone / tax | 903.8 | **A** | `rules/casting.rs:254`, `:2654-2663`, `:4691-4695` |

### 4.5 Combat (CR 508, 509, 510)

| choice | CR | class | site |
|---|---|---|---|
| Attacker set + attack targets | 508.1 | **A** | `rules/combat.rs:66-184` (validation), `:613-618` (recorded). Validation-only: the engine never computes the maximal requirement-satisfying set for the player |
| Attack requirements (goad, must-attack) | 508.1d / 701.15b | **A** *since PB-DP4* — **this row was originally rated A in error; the true pre-PB-DP4 class was D** | `rules/combat.rs:272-432`. **This row's original "A" was wrong**: the requirement-yields-to-restriction carve-out at `:412-424` covered only `CantAttackOwner`, never `CantAttackYouUnlessPay`, so CR 508.1d's *"that player is not required to pay that cost"* was violated and a forced attacker plus an unpayable tax on every viable opponent **deadlocked** the declare-attackers step (declaring is illegal on the tax check, omitting is illegal on the must-attack check). PB-DP4 added `has_uncosted_attack_target`, used by both the goad and the `MustAttackEachCombat` blocks, closing **OOS-RS3-4**. Residual: the goad *directional* check (`:336-374`) still has no cost carve-out — **OOS-DP4-3** |
| **Attack cost** (Propaganda) | **508.1c** restriction; **508.1h/i/j** payment (was mis-cited **508.1g**, which is *optional* "as it attacks" costs — the Exert row below) | **A** *since PB-DP4* | `rules/combat.rs:185-265` (cost computation) + the debit in the mutation section — see **DP-10**. The total is built as a real summed `ManaCost` per defending player and debited via `casting::pay_cost` (CR 508.1j), colour-correct, with restricted mana correctly excluded (CR **106.6**). Two residual deviations: no CR 508.1i mana-ability window (**OOS-DP4-2**) and a hybrid/Phyrexian tax is rejected rather than paid (**OOS-DP4-1**) |
| Enlist | 702.154a | **A** | validated `rules/combat.rs:449-539`, tapped `:602-612`, persisted `:634-636` |
| Exert | 701.43d | **A** | validated `rules/combat.rs:551-587`, applied `:637-649`, consumed by untap `rules/turn_actions.rs:1123-1125` |
| Blocker set | 509.1 | **A** | `rules/combat.rs:783-1095` — a thorough restriction set incl. menace at `:1067-1095` |
| Block requirements | 509.1c | **A**, narrow | only Provoke-style `forced_blocks` (`rules/combat.rs:1104-1305`); no generic "must block if able" / Lure |
| **Cost to block** | 509.1 | **D** | no `GameRestriction` variant, no `Command::DeclareBlockers` field |
| Damage assignment **order** | 509.2 | **A** if sent, **B** if not | honoured `rules/combat.rs:1387-1441`; fallback `:1485-1510` iterates `OrdMap` ⇒ **ascending blocker `ObjectId`**. **Nothing ever asks**: there is no `BlockerOrderRequired` event and no `LegalAction` |
| Damage assignment **amounts** | 510.1a-d | **B** | `rules/combat.rs:1536-1596` — exactly lethal to each non-last blocker, all remaining power dumped on the last; trample assigns the *minimum* lethal (max trample-through, `:1557-1576`). No command exists |
| Deathtouch lethal = 1 | 702.2b/c | **A** | `rules/combat.rs:1546-1547` |
| Attack a battle | 506.1 | **D** | `AttackTarget` (`crates/card-types/src/state/combat.rs:14-19`) has only `Player` and `Planeswalker` (Battle subsystem deferred) |
| Banding | 702.21 | **C** | zero occurrences in `crates/engine/src` or `crates/card-types/src`; no `KeywordAbility` variant, so nothing can silently misbehave. Marker: `docs/mtg-engine-ability-coverage.md:195` |

### 4.6 Triggered-ability targets (CR 603.3d)

**Class B.** `rules/abilities.rs:7174-7500`, inside `flush_pending_triggers`
(`abilities.rs:6950`). For `PendingTriggerKind::Normal` and `CardDefETB`, the ability's
`TargetRequirement`s are read from layer-resolved characteristics and satisfied by
first-match scan:

```rust
            // CR 603.3d: For CardDef-based triggered abilities (Normal / CardDefETB),
            // look up the target requirements from the ability definition and
            // auto-select legal targets using deterministic first-match fallback.
            // If any required target has no legal candidate, skip this trigger.
```
— `abilities.rs:7178-7181`

Exact rules by requirement kind:

- `TargetPlayer` / `TargetCreatureOrPlayer` / `TargetAny` / `TargetPlayerOrPlaneswalker`:
  **first non-lost, non-conceded opponent in `turn_order`, else the controller**
  (`abilities.rs:7249-7270`).
- `TargetOpponent`: first active opponent, **never** self-fallback (`:7276-7291`).
- Graveyard-card requirements: first match in `state.objects` iteration order, i.e.
  **ascending `ObjectId`** (`:7293-7362`).
- `UpToN`: contributes **zero** targets for a permanent inner requirement (`:7419-7422`).
- Battlefield permanents: first match in ascending `ObjectId` (`:7425+`), correctly filtered
  through `layers::expect_characteristics` and `validate_target_protection`
  (`:7434-7450` — CR 613.1f, so Humility is respected).

This is **CR-compliant**: protection/hexproof/shroud are honoured and a requirement with no
legal candidate removes the ability from the stack, exactly as CR 603.3d directs. It is not
a rules violation — it is a total loss of agency across **84 effectively-`Complete` defs**.
Modal triggers are the same shape: `abilities.rs:8398-8405` sets `modes_chosen = vec![0]`
(CR 603.3c), affecting 5 `Complete` defs.

**No hook exists.** The `Command` enum (`rules/command.rs:19-638`) has no trigger-target
variant, and `rules/events.rs` has no `TriggerTargetRequired` event.

### 4.7 Trigger stack ordering (CR 603.3b)

| choice | class | site |
|---|---|---|
| APNAP order **across** players | **A** (correctly deterministic; not a player choice) | `rules/abilities.rs:6963-6975`; `apnap_order` at `:8438-8452` rotates `turn_order` to start at the active player |
| Relative order **within** one player's batch | **B** | same `sort_by_key` — there is no secondary key and no second sort. `slice::sort_by_key` is stable, so same-controller triggers keep insertion order; insertion is always `push_back` over sweeps that iterate `state.objects` (an `OrdMap`). Effective rule: **ascending source `ObjectId`, then ability index, then sweep-call order** |

No `Command` and no `GameEvent` for trigger ordering exists.

### 4.8 Optional ("may") triggers and intervening-if (CR 603.4)

| choice | class | site |
|---|---|---|
| Costless "you may X" on a trigger | **D** | `AbilityDefinition::Triggered` (`crates/card-types/src/cards/card_definition.rs:338-365`) has **no** `optional` field. Cards are authored mandatory and marked `known_wrong` — e.g. `crates/card-defs/src/defs/consecrated_sphinx.rs:26-44`: `// Note: "you may" optional not in DSL — always draws`, `Completeness::known_wrong("'you may draw two cards' implemented as a mandatory draw")`. 19 defs carry such a marker |
| "May pay {cost}", tax shape | **C** | `Effect::MayPayOrElse` — `effects/mod.rs:3425-3428` discards `cost`/`payer` and always runs `or_else`. Gated; 0 `Complete` defs |
| "May pay {cost}", benefit shape | **B** | `Effect::MayPayThenEffect` — `effects/mod.rs:3432-3465` → `try_pay_optional_cost` (`:8531-8544`): **the only branch is affordability**, so the payer is force-fed life, sacrifices and discards whenever legal. Ungated; 11 `Complete` defs |
| "Counter unless controller pays" | **B** | `effects/mod.rs:3473-3483` — `cost` discarded, always counters. 7 `Complete` defs |
| Intervening-if at **resolution** | **A** | `rules/resolution.rs:2119-2135` re-evaluates the card-def `Condition` and removes the ability if it no longer holds |
| Intervening-if at **queue time** | **D** | Only two paths check it: ETB (`rules/replacement.rs:1446-1456`) and graveyard-zone triggers (`rules/abilities.rs:6910-6916`). `turn_actions.rs` and `combat.rs` contain zero occurrences of `intervening_if`. Self-documented at `rules/turn_actions.rs:264-266` and in the defs, e.g. `crates/card-defs/src/defs/loyal_apprentice.rs:21-30` |

The queue-time gap produces a **false-positive trigger**: condition false when the ability
would go on the stack, true at resolution ⇒ this engine fires, real Magic never triggers.
The false-negative direction is correctly handled by the resolution check.

### 4.9 Resolution-time choices

There is **no resolution-time decision channel at all**. `execute_effect_inner` returns `()`
and cannot suspend; `GameState` has six `pending_*` fields and none of them is an
effect-choice; the `Command` enum has no general `MakeChoice`.

| choice | CR | class | site & rule |
|---|---|---|---|
| Modal `Effect::Choose` | 700.2 | **C** | `effects/mod.rs:3419-3424` — always `choices.first()`; `prompt` inert. Gated; 0 defs use it |
| **Scry** keep-or-bottom | **701.22a** | **B** | `effects/mod.rs:3089-3098` — **every** scried card goes to the bottom, ObjectId-ascending. Scry is a self-mill-to-bottom in this engine; "keep on top" is unreachable. 16 `Complete` defs |
| **Surveil** keep-or-graveyard | **701.25a** | **B** | `effects/mod.rs:3123-3130` — **all** looked-at cards are milled. Surveil N ≡ Mill N. 8 `Complete` defs |
| Library search pick | 701.23 | **B** | `effects/mod.rs:3032` — `candidates.iter().min_by_key(\|&&id\| id.0)`, i.e. **lowest `ObjectId`** among filter matches. 74 `Complete` defs |
| Discard-as-effect | 701.8 | **B** | `effects/mod.rs:8611-8619` `discard_cards` — lowest `ObjectId` in hand, repeated. "Reveal your hand, opponent chooses" has no representation at all |
| `WheelDraw` (wheels: Windfall, Wheel of Fortune, …) | 701.8 | **A** — no choice exists | `Effect::WheelHand` (`card_definition.rs:2503-2511`) discards the player's **whole** hand — `effects/mod.rs:698` and `:730` both call `discard_cards(state, p, hand_size, events)` — so the lowest-`ObjectId` pick order in the row above is unobservable. The `WheelDraw` enum (`:2538-2550`) only sizes the redraw (`ThatMany` / `Fixed` / `GreatestDiscarded`) — a count, not a player choice. 10 defs. Listed because the task brief named it; there is nothing to hook |
| Sacrifice-as-effect / edicts | 701.21a | **B** | `effects/mod.rs:8193-8206` `sacrifice_permanents_for_player` — `n` lowest `ObjectId`s. Under `EachPlayer` this systematically takes each player's earliest-entering permanent |
| Cascade "you **may** cast" | 702.85a | **B** | `rules/copy.rs:366-368` — no decline branch; the `if` is only the legality test. The free-cast also gets `targets: vec![]` (`copy.rs:389`) and `modes_chosen: vec![]` ⇒ mode 0 (`copy.rs:430`) |
| Discover "you may cast" | 701.57 | **B** | `effects/mod.rs:3837-3848` — always casts |
| `PlayExiledCard` (Hideaway payoff) | 702.75a | **B** | `effects/mod.rs:4361-4364` — always plays |
| Hideaway "exile one of them" | 702.75a | **B** | `rules/resolution.rs:5903-5904` — top card always |
| Exploit "you may sacrifice" | 702.110a | **B** | `rules/resolution.rs:3853-3854` — **always declines**; TODO at `:3835` names the missing `Command::ExploitCreature` |
| Riot: counter **or** haste | 702.136a | **B** | `rules/resolution.rs:904` — always +1/+1 counter; TODO at `:905` names `Command::ChooseRiot` |
| Proliferate "choose any number" | 701.34a | **B** | `effects/mod.rs:3732` — auto-selects **all** eligible. The doc at `:3718-3731` flags that this can kill its own controller via poison (CR 704.5c). 25 `Complete` defs |
| `ChangeTargets` "you may choose new targets" | 115.7d | **B** | `effects/mod.rs:6562-6566` — always declines when optional; when `must_change` (115.7a) picks the smallest `ObjectId` without re-checking the original `TargetRequirement` |
| Bolster tie-break / Amass which Army | 701.29a / 701.47a | **B** | `effects/mod.rs:2624-2628`, `:2738-2744` — lowest `ObjectId` |
| `ChooseCreatureType` / `ChooseColor` | 106.12 / 614.12a | **B** | `effects/mod.rs:3660-3664`, `rules/replacement.rs:1747-1760` — most common among **the controller's own** battlefield permanents, which is the wrong optimisation for "choose, then destroy the rest" shapes |
| `LookAtTopThenPlace` `optional` flag | — | **B** | `effects/mod.rs:5076` destructures `optional: _`; the field is inert by construction. Winner picked lowest-`ObjectId` at `:5157`, and `place_cost` is paid even into a whiff (`:5100-5117`) |
| Coin flip / dice | 705.2 / 706.2 | **A** | `effects/mod.rs:3981-3997`, `:4002-4014` — seeded from `state.timestamp_counter`. Correctly deterministic for replay; not a choice |

### 4.10 Replacement-effect ordering (CR 616.1)

The zone-change path is genuinely correct and is the model the rest of the engine should
follow.

| choice | class | site |
|---|---|---|
| Ask-or-not condition | **A** | `rules/replacement.rs:82-122` `determine_action`. Exact rule: **ask iff `applicable.len() >= 2` AND (`self_ids.len() == 0` OR `self_ids.len() >= 2`)**. With ≥2 applicable and exactly one self-replacement it auto-applies — correct per CR 616.1a |
| Self-replacements first | **A** | `rules/replacement.rs:63-73` (CR 614.15/616.1a), reinforced by the two-pass ETB call order at `:1080-1107` |
| Correct chooser | **A** | `rules/replacement.rs:775-787` — affected object's **controller**, owner only as fallback; the comment explains why owner is wrong after a control change |
| Apply-one-then-recheck loop | **A** | `rules/replacement.rs:795-864`, `:954-978`, `:1034-1046` — CR 616.1f, with `already_applied` threading for CR 614.5 |
| Zone change genuinely **waits** | **A** | `rules/sba.rs:670-688` defers and pushes `PendingZoneChange`; `:529-533` / `:736-740` skip the object on later passes. Object-scoped, not game-scoped |
| Sender trust boundary | **A** | `rules/replacement.rs:139-193` — rejects a non-affected sender (`:163-172`) and any id not currently applicable via `find_applicable`, not mere existence (`:183-189`) |
| **ETB** replacement ordering | **B** | `rules/replacement.rs:1109-1120` — `NeedsChoice ⇒ choices.first()`. Tie-break: self-replacements first, then `state.replacement_effects` registration order. Defensible today (the shipped ETB modifications commute) but a CR 616.1 divergence in principle |
| Eight other `find_applicable` sites | **B** | Of 13 call sites only 5 route through `determine_action`. The rest apply **all** applicable effects in registration order and never ask. Two are semantically order-sensitive: `apply_damage_prevention` (`rules/replacement.rs:2387` — prevent-N vs redirect vs double interleave differently) and `apply_counter_replacement` (`:2878` — `DoubleCounters` before/after `HalveCounters` differ on odd counts). The other five commute |
| Regeneration vs umbra armor | **B** | `rules/sba.rs:546-558`, self-flagged MR-SR29-02; also `effects/mod.rs:1014` |
| **`WouldDraw` multi-replacement** | ~~**D**~~ → **A** | **SHIPPED (PB-DP5, `scutemob-153`).** There were **three** emit sites, not the two listed here: `rules/turn_actions.rs::draw_card`, the twin at `effects/mod.rs::draw_one_card` (renamed `draw_cards_for_player`) and a third the audit never named, `rules/replacement.rs::draw_card_skipping_dredge` (the post-dredge-decline path, reachable via `handle_choose_dredge(None)` and golden script `replacement/014`). All three now record a `GameState.pending_draws` entry; `handle_order_replacements` grows a draw arm; the answer completes the draw through the chosen order. See **DP-5** |

### 4.11 Cleanup and other turn-structure choices (CR 514.1)

| choice | CR | class | site |
|---|---|---|---|
| Hand-size discard | **514.1** | **B** | `rules/turn_actions.rs:1280-1293` — see **DP-3** |
| "Until end of turn" expiry | 514.2 | **A** | `rules/turn_actions.rs:1369-1370`; damage clear `:1348-1350`; pools `:1375` |
| No priority in cleanup | 514.3 | **A** | `crates/engine/src/state/turn.rs:63-65` |
| Extra cleanup round | 514.3a | **A** | `rules/engine.rs:1731-1762` + the non-advance guard at `:1678-1695`, with a 100-round cap |
| Echo / cumulative upkeep / recover pay-or-sacrifice | 702.30a / 702.24a / 702.59a | **A** plumbing, **A** enforcement *since PB-DP4* (with a stated CR 608.2d timing deviation) | `rules/engine.rs::force_resolve_overdue_payments`, hooked into `handle_all_passed`'s stack-empty branch — see **DP-11**. The three `pending_*` vectors are now consulted: an unanswered payment gets the CR 118.12a "didn't pay" branch at the first subsequent priority round that ends with an empty stack. Deviation: CR 608.2d wants the choice *during* the ability's effect; this engine defers it by one priority round (which is what preserves the CR 608.2g mana-ability window and keeps the choice a real one) |
| "You may choose not to untap" | 502.2 | **D** | `rules/turn_actions.rs:942-1150` untaps unconditionally; only static flags gate it |
| Vote / Council's Dilemma / Will of the Council | 701.36 | **D** | zero occurrences across `crates/engine/src` and `crates/card-types/src` |
| Legend rule: which to keep | 704.5j | **B** | `rules/sba.rs:960-965` — **highest** `ObjectId` is kept (most recently entered). Flagged MR-SR29-01 |

### 4.12 Priority itself (CR 117.3)

Not in the task's class list, but it is the decision every other decision hangs off, and it
is wrong. See **DP-1**.

---

## 5. Ranked B/D findings

One row — **DP-32** — is marked `A*`. Its *choice* is class A (honoured, never defaulted); it
is listed here because the surrounding machinery is not, and a reader ranking work items needs
to see it next to the rest. It is the only non-B/D row in this section.

Ranking dimension is **human-play impact**: how often a human at one seat of a 4-player
Commander game hits the site × how far the engine's pick is from what a player would choose ×
whether it ships in `Complete` cards or in core rules.

### Tier 0 — correctness class: the engine's behaviour diverges from the CR

Four of the five are class **D** — wrong, not merely un-consulted. **DP-3 is class B**, and
sits here because it is the only decision in the whole audit with *no `Command` at all* on a
turn-based action every human hits.

Reachability differs from severity and the two are deliberately not the same list: DP-1, DP-2
and DP-3 are core-reachable (no card required), while DP-4 needs one of three specific
`Complete` cards in the deck and DP-5 needs two `WouldDraw` replacements on the board. The
core-reachable set is §3.2's six.

**Correction (PB-DP5, `scutemob-153`):** DP-5's reachability was in fact **zero**, not merely
narrow — **no card in the 1,804-def corpus registers a `WouldDraw` replacement at all**, so
"needs two on the board" could never be satisfied from a legal deck. That does not change its
Tier-0 placement (the defect is in engine machinery every future draw-replacement card would
route through, and its planning surfaced **OOS-DP5-7**, which *is* live), but it is a caution
about this table's method: "which cards does it need" was answered from the finding's shape
rather than by enumerating the corpus, and the enumeration was cheap.

| id | class | finding | CR | site |
|---|---|---|---|---|
| **DP-1** | D | **SHIPPED (PB-DP1, `scutemob-149`).** Priority after casting / activating / a special action goes to the ACTOR, not the active player. CR 117.3c: "If a player has priority when they cast a spell, activate an ability, or take a special action, **that player** receives priority afterward." The comment at `casting.rs:4712` misquoted CR 601.2i as "Then the active player receives priority" (it actually reads "If the spell's controller had priority before casting it, they get priority"); `abilities.rs:1384` cited **CR 602.2e, which does not exist** (fixed to CR 602.2b). Verified breakdown of the "~20 sites" estimate: **14 Group-A sites** (of which 6 actually flip behaviour: `casting.rs:handle_cast_spell`, `abilities.rs::handle_activate_ability`/`handle_cycle_card`/`handle_activate_bloodrush`/`handle_ninjutsu`/`handle_crew_vehicle`; the other 8 are AP-gated identity writes), **3 Group-B sites ruled no-change** (`engine.rs:757/958/1072` — echo/cumulative-upkeep/recover reassign priority at resolution time, when no player holds it; comment-only fix, seeded OOS-DP1-1), **8 Group-D sites** (2 handlers already correct by construction, 4 missing the CR 117.4 `players_passed` reset, 3 missing the CR 116.3/117.3c grant — a fix-cycle review pass added the missing entry priority guard to all 3, closing part of OOS-DP1-2), and **5 confirmed false positives** that were never DP-1 (`engine.rs:1759`, `:1805` and `combat.rs:1373` are CR 117.3a turn-based-action/step-start grants; also two `handle_activate_loyalty_ability`/`handle_level_up_class` sites the original roster missed entirely). | 117.3c, 601.2i | `rules/casting.rs:4712-4715`; `rules/abilities.rs:1384-1387`, `:1552`, `:1753`, `:1967`, `:2102`, `:2341`, `:2504`, `:2681`, `:2857`, `:8791`, `:9000`, `:9202`; `rules/engine.rs:1461` (craft), `handle_turn_face_up`, `handle_activate_loyalty_ability`, `handle_level_up_class` |
| **DP-2** | D | **SHIPPED (PB-DP2, `scutemob-150`).** **A mulligan is a content no-op, and `cards_to_bottom` goes to the library's TOP.** `handle_take_mulligan` moves every hand card to the library — `Zone::insert` is `push_back` and `Zone::top()` is `v.last()`, so they land on top — emits a **phantom** `GameEvent::LibraryShuffled` with no permutation, then draws 7. **The same seven cards return, reversed.** Separately, `handle_keep_hand` bottoms cards with `move_object_to_zone` (top) instead of `push_front` (bottom), so the cards you bottom are the next cards you draw. This is the OOS-RS-1 top/bottom inversion class that PB-RS1 swept — the mulligan was not in its roster. Tests (`crates/engine/tests/rules/commander.rs:1400-1495`) assert hand counts and events only, never library position. **Both halves are now fixed**: `handle_keep_hand` bottoms with `move_object_to_bottom_of_zone` (`push_front`), so `cards_to_bottom` index 0 ends up ABOVE later entries and the pre-existing library — including its top card — is untouched; `handle_take_mulligan` runs a real seeded Fisher-Yates `Zone::shuffle` (seeded from the existing `state.timestamp_counter`, the MR-M7-17 idiom) after the hand→library moves and **before** both the `LibraryShuffled` event and the 7 draws, so the event is no longer phantom (Architecture Invariant 4). 4 fail-before/pass-after probes; PROTOCOL 27 / HASH 63 unmoved. Closes **OOS-M11-1**, widened per §7 to cover the `handle_keep_hand` half. **CR correction**: this row's original cite of `103.4b` was stale — live CR 103.4b is the *Vanguard starting life total*; both the shuffle and the bottoming live in a single sentence of CR 103.5, with 103.5c supplying the multiplayer free-first-mulligan adjustment (the engine's own source comments already cited 103.5 correctly). | 103.5, 103.5c | `rules/commander.rs:808-848` and `:886-890`; `crates/card-types/src/state/zone.rs:109`, `:159-164`, `:187`; `state/builder.rs:286` |
| **DP-3** | B | **Cleanup discard has no `Command` at all** and auto-picks the **highest `ObjectId`** in hand — the most recently drawn card — one at a time. Hand is `Zone::Unordered` (an `OrdSet`), `object_ids()` yields ascending, `.last()` takes the top. Madness is correctly honoured on this path (`:1301-1342`), which means the auto-picker can involuntarily fire Madness on a card the player would never have chosen. | 514.1 | `rules/turn_actions.rs:1280-1293`; `crates/card-types/src/state/zone.rs:130-135` |
| **DP-4** | ~~D~~ → **A** | **SHIPPED (PB-DP3, `scutemob-151`).** The fix is a **validation lift**, not a bolted-on empty-check: the range / duplicate / `min_modes` / `max_modes` checks now run whenever the object is modal, so it also closes the auto-select across the other **37** `min_modes: 1` defs — the headline below understated the scope to the 3 commands. `rules/abilities.rs`'s activated-ability twin (§4.2) shipped in the same PB. A narrow CR 702.120a exemption keeps escalate's backward-compat path alive with its derived count bounds-checked (**OOS-DP3-1**); `resolution.rs`'s `vec![0]` fallback is **retained** because four free-cast producers build Spell stack objects without calling `handle_cast_spell` (`copy.rs:386` cascade, `copy.rs:614` discover, `resolution.rs:5167` cipher, `resolution.rs:5837` suspend — **OOS-DP3-3**). §8's "mirror the Spree guard" prescription was deliberately **not** followed: the Spree guard was kept (it fires earlier and owns the CR 702.172a message). **No wire change — PROTOCOL 27 / HASH 63 unmoved.** Blast radius: 3 test lines, 2 golden scripts, 1 harness line, the simulator/TUI callers, one SR-15 registry declaration, **0 card-def edits**. Original finding: **An empty `modes_chosen` bypasses `min_modes` entirely.** The range / duplicate / `min_modes` / `max_modes` checks live only inside the `!modes_chosen.is_empty()` branch. Cast-time target slicing then assumes `vec![0]` (`casting.rs:3645-3653`) and resolution re-derives `vec![0]` (`resolution.rs:335-341`). **Cryptic Command, Austere Command and Incendiary Command all declare `min_modes: 2, max_modes: 2` and all three are `Complete`** — cast one with no modes and it pays the full cost and resolves *one* mode, silently. The Spree path *does* hard-reject empty modes (`casting.rs:2940-2944`); the general modal path has no equivalent. | 601.2b, 700.2a | `rules/casting.rs:3506`, `:3536-3549`, `:3555-3559`, `:3645-3653`; `rules/resolution.rs:335-341`; `crates/card-defs/src/defs/cryptic_command.rs:31-32`, `austere_command.rs:27-28`, `incendiary_command.rs:37-38` |
| **DP-5** | ~~D~~ → **A** | **SHIPPED (PB-DP5, `scutemob-153`).** Original finding: **the `WouldDraw` multi-replacement prompt is unanswerable and the draw is destroyed.** `draw_card` emitted `ReplacementChoiceRequired` and returned early recording **no** pending state — there was no draw-pending field on `GameState` at all — while `handle_order_replacements` required a matching `pending_zone_changes` entry, so the `Command::OrderReplacements` the player was being asked for was rejected and the draw could never complete. **FIXED** by a new `GameState.pending_draws: Vector<PendingDraw>` (`{ player, already_applied, remaining, sets_has_drawn_for_turn }`, sorted `already_applied` for SR-9b determinism), recorded at **all three** emit sites, plus a `resolve_pending_draw` mirroring `resolve_pending_zone_change`'s CR 616.1f re-check. `handle_order_replacements` routes between a pending zone change and a pending draw **by applicability**, which is total rather than heuristic: `trigger_matches` only matches same-variant `ReplacementTrigger` pairs and ends `_ => false`, so the two candidate sets are provably disjoint and the tie-break is unreachable. Both SR-29 security checks survive on the draw arm (the sender must be the affected chooser; every ordered id must be *currently* applicable via `find_applicable`, not merely registered). **Three corrections to this row's own premises.** (1) **"Reachable with any two `WouldDraw` replacements" overstates the card exposure to zero**: **no card in the 1,804-def corpus registers a `WouldDraw` replacement** (the only corpus hit is an `inert` completeness *note* in `out_of_the_tombs.rs:32`), so DP-5 was unreachable from a legal deck and **0 card defs were edited**. The honest framing is "class-D engine defect + the precondition for ever authoring the `WouldDraw` family", not "live-wrong on a `Complete` def today". (2) **The damage was worse than filed**: because the draw *sequence* kept iterating after the deferral, `Effect::DrawCards { count: 3 }` emitted **three** unanswerable prompts and drew **zero** cards. `draw_cards_for_player` now owns the sequence and breaks on deferral per CR **614.11a**, stashing the remainder in `PendingDraw.remaining` for the resume to finish. (3) The existing test cited here (`tests/rules/replacement_effects.rs`) asserted only deferral and is now strengthened in place to assert the draw completes through the **chosen** order. **Deviations, stated:** a deferred draw does **not** suspend the rest of the effect — "draw three, then discard three" runs its second half against a hand that does not yet hold the drawn cards (**OOS-DP5-5**); the resume does not re-offer dredge on draws 2..N (**OOS-DP5-3**); a "draw 3 from an empty library" now makes one attempt rather than three (CR 121.3, outcome-identical — one `PlayerLost` instead of three); and nothing gates priority, SBAs or step advancement on `pending_draws`, so an unanswered entry leaves the game exactly as playable as before (hard constraint: no hang; **OOS-DP5-2** owns the eventual deadline). **The review's stress test found one real defect in the first implementation and it was fixed at the source**: the runner's claim that a single re-check is equivalent to the CR 616.1f loop is **false** in `determine_action`'s CR 616.1a branch — `AutoApply` is returned with 2+ effects still applicable when exactly one is a self-replacement, and a non-`SkipDraw` self-replacement then returned `Proceed`, performing the draw and silently dropping the remainder *including a `SkipDraw` that CR 616.1f says must then apply*. `check_would_draw_replacement` now runs its own bounded re-check, mirroring the sibling `WouldChangeZone` path. **§8's wire prediction is CONFIRMED: HASH 63 → 64** (forced by the gate, not hand-bumped; appended v64 `HASH_SCHEMA_HISTORY` row, no existing row edited), **PROTOCOL 27 unmoved**, no new `Command` / `GameEvent` / `Effect` variant — `Command::OrderReplacements` was reused as §8 predicted. 17 tests (13 + T10b + T14 + T15 + 1 strengthened), tests 3,781 → **3,797**. Seeds **OOS-DP5-1..8** filed below; the largest is **OOS-DP5-7**, a *live* free-card exploit on `Command::ChooseDredge` found while planning this PB and arguably higher-severity than DP-5 itself. | 616.1, 616.1a, 616.1f, 614.11, 614.11a (+ 121.3, 702.52a) | `rules/replacement.rs::{check_would_draw_replacement, perform_one_draw, resolve_pending_draw, handle_order_replacements, draw_card_skipping_dredge}`; `rules/turn_actions.rs::draw_card`; `effects/mod.rs::draw_cards_for_player`; `crates/card-types/src/state/replacement_effect.rs::PendingDraw`; `state/{mod,builder,hash}.rs` |

### Tier 1 — silently wrong in most games, driven by `Complete` cards

| id | class | finding | CR | `Complete` defs | site |
|---|---|---|---|---:|---|
| **DP-6** | B | Triggered-ability targets auto-selected by first-match (**OOS-M11-4**) | 603.3d | **84** | `rules/abilities.rs:7174-7500` |
| **DP-7** | B | Library search picks the lowest `ObjectId` — every tutor fetches for you | 701.23 | **74** | `effects/mod.rs:3032` |
| **DP-8** | B | Scry sends **all** cards to the bottom; "keep on top" is unreachable | 701.22a | 16 | `effects/mod.rs:3089-3098` |
| **DP-9** | B | Surveil sends **all** cards to the graveyard; Surveil N ≡ Mill N | 701.25a | 8 | `effects/mod.rs:3123-3130` |
| **DP-10** | **SHIPPED (PB-DP4, `scutemob-152`)** — was D | **Propaganda attack tax is checked but never charged.** The mana pool is inspected once (`combat.rs:250-253`) and never debited anywhere in the 600-line handler. Float the mana, attack free, keep the mana. Colour is also flattened to generic (`:218-227`). **FIXED**: the tax is now a real summed per-defender `ManaCost` debited in the mutation section (CR order 508.1f tap → 508.1h total → 508.1j pay); colour is preserved; **restricted mana no longer counts toward affordability** (CR **106.6** — `can_pay_cost`/`pay_cost` are called with `spell: None`, resolving the `total_with_restricted()`-vs-`spend(cost, None)` inconsistency in the strict direction; every `ManaRestriction` variant is spell-scoped, so this is correct and is a deliberate behaviour flip); a hybrid/Phyrexian/X tax is **rejected** rather than silently dropped (**OOS-DP4-1**; pre-PB-DP4 the pips summed to 0 and the attack was free — the OOS-RS-2 class). `GameEvent::ManaCostPaid` is reused, so **no wire change**. The in-code claim that interactive payment *"requires a new `DeclareAttackers` command field"* is **falsified and deleted** — the tax derives entirely from `state.restrictions` + the declared attackers. **CR cite corrected: 508.1g → 508.1c/h/i/j.** **CR 508.1d honoured**, closing **OOS-RS3-4**. `propaganda.rs` and `ghostly_prison.rs` (both `Complete`) stop being live-wrong with **0 def edits** | ~~508.1g~~ **508.1c / 508.1h/i/j** | — | `rules/combat.rs:185-265` + debit site |
| **DP-11** | **SHIPPED (PB-DP4, `scutemob-152`)** — was D | **Echo / cumulative upkeep / recover never enforce the "otherwise, sacrifice".** `resolution.rs:2785` asserts "The game pauses until a `Command::PayEcho` is received"; **no code implements that pause.** The three `pending_*` vectors are read only inside their own handlers — never by priority, SBA or step advancement. Pass priority and the permanent is neither paid for nor sacrificed. Compounding: none of the three has a `LegalAction`, so in a bot/M11 game the command is never sent. **FIXED** — and the design decision is the substance of this row: the vectors are now consulted by `force_resolve_overdue_payments`, called from `handle_all_passed`'s **stack-empty** branch, which applies the CR **118.12a** "didn't pay" branch to every unanswered payment before the game leaves that priority round. **§8's phrasing — "wire: none if the 'otherwise' is applied at resolution rather than gated on priority" — is confirmed, but the boundary chosen is neither of those two**: it is *the end of the priority round after* resolution, because (a) applying it **at** resolution destroys the CR 608.2d/608.2g choice and makes `Command::PayEcho` unreachable (contradicting the LegalAction requirement), and (b) **gating priority deadlocks** any seat that never sends the command — `driver.rs` answers a rejected command with a silent `PassPriority`, so a refused pass is an infinite retry with no error, strictly worse than the bug. Auto-**decline**, never auto-pay (CR 118.12a; auto-pay is DP-19's bug class and would spend mana the player was saving). APNAP order (CR 101.4), entries snapshotted before any handler mutates the vector. Extra-round-not-advance so dies-triggers land in the correct step (the CR 514.3a pattern); the extra round is guarded on `!payment_events.is_empty()` so an all-no-op sweep falls through. **Accepted deviation from CR 608.2d**: within the deferral window the permanent still exists (and a recover card still sits in its graveyard) though the CR would already have sacrificed/exiled it — it can be tapped for mana, sacrificed to another cost, targeted or blocked with; the *outcome* is not deviated from. Three `Complete` defs (`mogg_war_marshal`, `avalanche_riders`, `grim_harvest`) stop being live-wrong with **0 def edits**; `mystic_remora`'s `known_wrong` note becomes accurate. Residual: the deadline is postponable by keeping the stack non-empty (**OOS-DP4-12**). **PROTOCOL 27 / HASH 63 unmoved** | 702.30a, 702.24a, 702.59a (+ 118.12a, 608.2d/g, 101.4) | — | `rules/engine.rs::force_resolve_overdue_payments` + `handle_all_passed` stack-empty branch |
| **DP-12** | D | Costless "you may" on a trigger has **no DSL representation**; such cards are authored mandatory and marked `known_wrong` | 603.4 | 19 marked | `card_definition.rs:338-365`; `consecrated_sphinx.rs:26-44` |
| **DP-13** | B | Combat damage: amounts fully auto-computed (min-lethal, dump-the-rest-on-the-last, max trample-through); assignment order defaults to ascending blocker `ObjectId` and **nothing ever prompts** for `OrderBlockers` | 509.2, 510.1a-d | — | `rules/combat.rs:1485-1510`, `:1536-1596` |

### Tier 2 — real, narrower

| id | class | finding | CR | site |
|---|---|---|---|---|
| **DP-14** | B | Same-controller trigger ordering is a stable sort ⇒ ascending source `ObjectId`; no `Command`, no event | 603.3b | `rules/abilities.rs:6963-6975` |
| **DP-15** | D | Intervening-if checked at resolution but **not at queue time** except ETB and graveyard paths ⇒ false-positive triggers | 603.4 | `rules/turn_actions.rs:264-266`; `rules/replacement.rs:1446-1456`; `rules/abilities.rs:6910-6916` |
| **DP-16** | B | Edicts / sacrifice / discard effects pick lowest `ObjectId` (23 + 11 `Complete` defs) | 701.21a, 701.8 | `effects/mod.rs:8193-8206`, `:8611-8619` |
| **DP-17** | B | Proliferate auto-selects **all** eligible — can kill its own controller via poison | 701.34a | `effects/mod.rs:3732` |
| **DP-18** | B | `CounterUnlessPays` always counters (7 `Complete` defs) | 118.12a | `effects/mod.rs:3473-3483` |
| **DP-19** | B | `MayPayThenEffect` always pays when able (11 `Complete` defs) | 118.12 | `effects/mod.rs:3432-3465`, `:8531-8544` |
| **DP-20** | B | Cascade / Discover / `PlayExiledCard` always cast; cascade free-cast also gets no targets and mode 0. **PB-DP3 cross-reference (OOS-DP3-3):** four producers build `StackObjectKind::Spell` with `modes_chosen: vec![]` *without* calling `handle_cast_spell`, so PB-DP3's cast-time mode guard cannot reach them and `resolution.rs`'s `vec![0]` fallback must stay live until DP-20 is closed — `copy.rs:386` (cascade), `copy.rs:614` (discover), `resolution.rs:5167` (cipher copy), `resolution.rs:5837` (suspend). Whoever closes DP-20 owns retiring that fallback | 702.85a, 701.57 | `rules/copy.rs:366-368`, `:389`, `:430`; `effects/mod.rs:3837-3848`, `:4361-4364` |
| **DP-21** | D | Loyalty abilities: no priority check, no active-player check, no split-second check | 606.3, 702.61a | `rules/engine.rs:2451-2660` |
| **DP-22** | B | 8 of 13 `find_applicable` sites bypass `determine_action`; `apply_damage_prevention` and `apply_counter_replacement` are order-sensitive | 616.1 | `rules/replacement.rs:2387`, `:2878` |
| **DP-23** | D | Craft materials accept **any** player's battlefield permanents and graveyard cards | 702.167a | `rules/engine.rs:1357-1366` |
| **DP-24** | D | Accepted-and-discarded inputs: `ChooseDungeonRoom.room` bound to `_`; `CastSpellData.face_down_kind` bound to `_` and re-derived | 309.5a, 702.37c | `rules/engine.rs:526-530`; `rules/casting.rs:67` vs `:4661-4683` |
| **DP-25** | B | Fixed-branch resolution picks: Riot → counter; Exploit → decline; Hideaway → top card; ring-bearer → lowest `ObjectId`; legend rule → highest `ObjectId`; `ChangeTargets` → unchanged | various | §4.9, §4.11 |
| **DP-26** | B | Cost-payment micro-choices: generic pool spend order fixed (C→G→R→B→U→W, restricted first); convoke pip assignment colored-first WUBRG; forage prefers Food; `ExileOtherGraveyardCards` takes the `n` lowest | 601.2h, 702.51a, 701.61a | `player.rs:203-244`; `casting.rs:5493-5522`, `:3944-3946`; `abilities.rs:1162-1214` |
| **DP-27** | D | Target **division** (CR 601.2d) unimplemented end-to-end — no command field, no `Effect`, no resolution path | 601.2d | `fire_covenant.rs:25-37` |
| **DP-28** | D | Absent subsystems: Vote / Council's Dilemma (701.36); "you may choose not to untap" (502.2); cost-to-block; Battle as an `AttackTarget`. Banding is **C** (no enum variant, deliberately deferred) | 701.36, 502.2, 509.1 | §4.5, §4.11 |
| **DP-29** | D | `ActivateForecast` targets stored with **zero** validation; `ActivateBloodrush` target skips protection/hexproof/shroud | 601.2c | `rules/abilities.rs:1725-1748`, `:1845-1880` |
| **DP-30** | D | Only `spell_additional_costs[0]` is validated despite the comment claiming all are | 118.8 | `rules/casting.rs:3307-3309` |
| **DP-31** | B | Commander hand/library redirect (CR 903.9b) registered unconditionally ⇒ always opts in; the owner's "may" is dropped | 903.9b | `state/builder.rs:1199-1229` |
| **DP-32** | A* | Commander graveyard/exile choice (CR 903.9a) is honoured but **does not gate progress**; and it is modelled as a post-move SBA, not a replacement, so `CreatureDied` fires and CR 400.7 makes the commander a new object first | 903.9a | `rules/commander.rs:410-420`; `state/builder.rs:1181-1184` |

---

## 6. What the audit did **not** cover

Stated so a re-audit knows where the edges are.

- **Card-by-card verification.** The corpus counts in §3.1 are a regex sweep, not the serde
  walk `effect_choose_gate.rs` uses. Treat them as magnitudes, not exact rosters.
- **The simulator's `StubProvider` gaps** (Adventure, alt-costs, modes, convoke/improvise/delve)
  are move-generator limits, not engine decision-surface gaps, and are already catalogued as
  M11-local risk R4.
- **Whether each class-B tie-break is the *worst* choice.** This audit establishes what the
  engine picks; it does not model how much equity a player loses. That is the ranking input
  §5 approximates by frequency and by "distance from a sensible pick".
- **Deferred subsystems** already tracked elsewhere: Battle (OOS retriage plan), Banding
  (ability-coverage doc), P2P/hidden-information (M10).

---

## 7. OOS-M11 seed assessment

Explicitly requested. **`memory/primitives/rider-seed-triage-2026-07-19.md` was not modified**
— that queue is paused at PB-RS4 and belongs to the coordinator.

### OOS-M11-1 — mulligan no-shuffle: **CONFIRMED as filed; covers one of two defects**

The seed itself is exact. The **§8 risk table, row R2** — the row that proposes the id
`OOS-M11-1`, at `m11-session-plan.md:800` as committed at the time of this audit and `:814` in
that worktree's working copy, so grep for `OOS-M11-1` rather than trusting the line — reads
"A mulligan today returns the same hand — a live-wrong rules path (CR 103.5 requires a
shuffle)", and this audit confirms it verbatim. (The plan's §1 fact 1 hedges to
"near-identical"; the seed does not, and the seed is right.)

`Zone::insert` on an ordered zone is `push_back`
(`crates/card-types/src/state/zone.rs:109`) and `Zone::top` is `v.last()` (`:159-164`), so the
seven cards moved hand→library are the seven cards drawn back. No RNG is invoked — the engine
*does* have a deterministic seeded PRNG (`effects/mod.rs:8697-8703`, `:3049`, `:3148`, seeded
from `state.timestamp_counter`), and `handle_take_mulligan` calls none of it. The emitted
`GameEvent::LibraryShuffled` is a phantom, which is an Architecture Invariant 4 problem in its
own right ("events are the single source of truth for what happened").

**Widen the seed.** What the seed does *not* cover is a second, independent, cheaper defect on
the same command pair — this audit's genuinely new contribution here: `handle_keep_hand`
(`rules/commander.rs:886-890`) puts `cards_to_bottom` on the **top** of the library. That half
is a one-line `push_front` fix with no wire impact and it is a strictly-wrong rules path today.
Recommended rank: **Tier 0**, above every RS-queue item, on the same "live-wrong on a
`Complete` path" criterion that put OOS-RS-1 at the head of the RS queue.

**CLOSED by PB-DP2 (`scutemob-150`, 2026-07-26) — including the widening this audit
recommended.** Both halves shipped together in `rules/commander.rs`: `handle_take_mulligan`
now runs a real seeded Fisher-Yates `Zone::shuffle` before the `LibraryShuffled` event and
the draws (so the event is no longer a phantom), and `handle_keep_hand` now uses
`move_object_to_bottom_of_zone` (`push_front`) so bottomed cards land on the bottom. The seed
is `state.timestamp_counter` — the same deterministic PRNG source this section pointed at —
so replay determinism (SR-9b) holds and **no wire change was needed** (PROTOCOL 27 / HASH 63
unmoved), falsifying §8's HASH-bump prediction for the (b) half. 4 fail-before/pass-after
probes in `crates/engine/tests/rules/commander.rs`; tests 3,721 → 3,725.

### OOS-M11-2 — mana solver: **CONFIRMED, correctly ranked low, and it should not enter the primitive queue**

Both halves verified. `crates/simulator/src/mana_solver.rs` contains **zero** references to
`mana_pool` (so a human who taps manually and then casts is over-tapped), and it reads
`obj.characteristics.mana_abilities` directly (`mana_solver.rs:44`) rather than
`calculate_characteristics`, so a Cryptolith-Rite-granted or animated-land mana ability is
invisible to it.

But this is **simulator-side**, and the engine's own payment paths are layer-correct and
authoritative — `solve_mana_payment` returns a `Vec<Command>` of real `TapForMana` values that
`process_command` then judges. It cannot produce a wrong game state; it produces a wrong
*suggestion*. It belongs in M11-local (Session 3 item 7 already fixes the pool half), **not**
in the primitive queue, which is for engine correctness. Recommended rank: below every class-D
finding in §5.

> **Rider added by PB-DP4 (`scutemob-152`).** The attack-tax debit makes the pool-blindness
> cost real in combat too: a bot that taps for the tax and then has that mana actually *spent*
> will over-tap on its next action, because the solver never reads the pool. Still an
> M11-local Session 3 item, not a primitive-queue item — but it now costs the bot mana rather
> than only a suggestion. See also **OOS-DP4-8** (the provider offers attack targets whose tax
> the player cannot pay at all).

### OOS-M11-4 (né OOS-M11-3; renumbered — ID taken by scutemob-147's fuzzer-nondeterminism seed) — triggered-ability targets: **CONFIRMED; reclassify B, not D**

Verified at `rules/abilities.rs:7174-7500`. The fallback is **CR 603.3d-compliant**: it uses
layer-resolved characteristics (`layers::expect_characteristics`, `:7434`), honours
protection/hexproof/shroud (`validate_target_protection`, `:7438-7450`), never self-targets a
`TargetOpponent` requirement (`:7272-7291`), and removes the trigger from the stack when a
required slot has no legal candidate — exactly what CR 603.3d directs.

So this is not a rules violation. It is a **total loss of agency on 84 effectively-`Complete`
defs** — the largest single-site agency loss in the corpus after tutors. Recommended rank:
**top of Tier 1**, immediately below the Tier-0 correctness items, and it is the flagship
motivator for the pending-decision work in §8 because it is the first finding that *requires* a
new `Command` (and therefore a PROTOCOL bump).

---

## 8. Proposed primitive-queue insertions

Proposals only. The RS queue is paused at PB-RS4 and this audit does not edit it; these are
offered for the coordinator to rank against RS4..RS11.

Ordered correctness-first, matching the RS queue's own convention that a live-wrong path on a
`Complete` card outranks everything else.

| proposed | finding | wire impact | why here |
|---|---|---|---|
| **PB-DP1** | **DP-1** — priority holder after cast / activate / special action (CR 117.3c) — **SHIPPED** (`scutemob-149`) | **none** | ~20 mechanical sites, no new type (verified breakdown: 14 Group A / 3 Group B / 8 Group D / 5 false positives, see §5 DP-1). Highest correctness-per-line ratio in the whole audit, and it is the precondition for a human seat behaving like a player rather than a spectator. The original site list also missed `handle_activate_loyalty_ability` and `handle_level_up_class` — both fixed. A fix-cycle review pass (0 HIGH / 3 MEDIUM / 8 LOW) added an entry priority guard to the three Group-D "grant" handlers (`handle_turn_face_up`, `handle_activate_loyalty_ability`, `handle_level_up_class`), closing part of OOS-DP1-2; the CR 606.3 "their own turn" sorcery-timing gap on loyalty/level-up remains open (DP-21's scope) |
| **PB-DP2** | **DP-2** — mulligan. Split: (a) `cards_to_bottom` → `push_front`; (b) real shuffle — **SHIPPED** (`scutemob-150`) | **none** — the "(b) needs a seed on `GameState` ⇒ **HASH bump**" prediction is **falsified**: (b) reuses the existing `timestamp_counter` seed source (`effects/mod.rs:8697-8703`), so no `GameState` field was added and **PROTOCOL 27 / HASH 63 are unmoved** | (a) is a one-liner and closes a live-wrong path today. (b) is the OOS-M11-1 seed proper. **The split was unnecessary**: (b) did **not** need to trail (a) — with no wire cost, both halves shipped together as a 2-edit PB in one file. M11-local Session 2's pregame `redeal` is therefore no longer load-bearing for correctness |
| **PB-DP3** | **DP-4** — `min_modes` floor when `modes_chosen` is empty — **SHIPPED** (`scutemob-151`) | **none** — prediction **confirmed**: PROTOCOL 27 / HASH 63 unmoved, no `Command`/`GameEvent`/`Effect` variant and no `GameState` field touched | 3 `Complete` cards resolve half a spell at full price. **The "mirror the Spree guard at `casting.rs:2940-2944`" prescription was deliberately not followed**: the Spree guard was kept intact (it fires earlier, during cost computation, and owns the CR 702.172a message that `spree.rs:854` asserts), and the general fix is a **lift** — the range/duplicate/`min_modes`/`max_modes` checks moved out of the `!modes_chosen.is_empty()` gate so they run whenever the object is modal. That made the yield larger than the row predicted: **37 more `min_modes: 1` defs** plus the activated-ability twin (§4.2) were fixed alongside the 3 commands, for **0 card-def edits**. Escalate keeps a narrow CR 702.120a exemption (**OOS-DP3-1**) |
| **PB-DP4** | **DP-10** + **DP-11** — attack tax debit; echo/cumulative-upkeep/recover enforcement — **SHIPPED** (`scutemob-152`) | **none** — prediction **confirmed**: PROTOCOL 27 / HASH 63 unmoved, no new `Command`/`GameEvent`/`GameState` variant or field. The existing `GameEvent::ManaCostPaid` carries the tax debit and the three `Pay*` commands + three `*PaymentRequired` events already existed. The row's conditional ("*if* the 'otherwise' is applied at resolution rather than gated on priority") held, though **the boundary actually chosen is neither** — see §5 DP-11 | Two "the cost is checked but never collected" bugs of the same shape. **The bundling rationale held**, and the shared lesson is sharper than the row predicted: *an affordability check is not a payment*, and both fixes amount to **making the check and the payment the same predicate** — DP-10 by calling `can_pay_cost`/`pay_cost` on one summed `ManaCost` instead of comparing a flattened `u32` against `total_with_restricted()`, DP-11 by making the code that *created* the pending entry and the code that *consumes* it reachable from the same control flow. Yield larger than filed: **5 `Complete` defs are live-wrong today and are made right with 0 card-def edits** (`propaganda`, `ghostly_prison`, `mogg_war_marshal`, `avalanche_riders`, `grim_harvest`). Also closed **OOS-DP1-1** (by deletion — the three priority bodges) and **OOS-RS3-4** (CR 508.1d must-attack-vs-cost), and found two bugs the audit had not filed: an unguarded life subtraction in the cumulative-upkeep `Life` arm (CR 119.4) and a mis-rated §4.5 "**A**" row. Tests 3,747 → **3,781** |
| **PB-DP5** | **DP-5** — `WouldDraw` pending-choice state — **SHIPPED** (`scutemob-153`) | **HASH 63 → 64** — prediction **CONFIRMED**, and it is the first PB-DP to move a version constant. The bump was forced by the gate (`declaration_fingerprint_is_pinned`), not hand-applied; the v64 `HASH_SCHEMA_HISTORY` row is appended with freshly-computed fingerprints and no existing row was edited. **PROTOCOL 27 unmoved**: `Command::OrderReplacements` was reused exactly as this row predicted, so no `Command` / `GameEvent` / `Effect` variant was added and `ReplayLog` (a `{ u8, Vec<Command> }` that never embeds `GameState`) keeps the new field off the wire | The engine asked a question it could not accept an answer to and silently ate a draw. **The row's implicit premise about yield needs correcting, and §5 does so: card yield is 0** — no def in the corpus registers a `WouldDraw` replacement, so this is a class-D engine defect and the precondition for authoring that family, not a live-wrong `Complete` card. Scope also grew by one site the audit never named (`draw_card_skipping_dredge`) and by the CR 614.11a sequence bug the finding did not mention (`DrawCards { count: 3 }` emitted **three** unanswerable prompts and drew **zero**). Its planning turned up **OOS-DP5-7**, a live free-card exploit on `Command::ChooseDredge` of the same trust-boundary class — deliberately deferred, and a strong candidate to rank next |
| **PB-DP6** | **DP-15** — intervening-if at queue time (CR 603.4) | **none** | Already accepted as a known limitation in the defs; closing it retires a whole class of def-level caveats |
| **PB-DP7** | **DP-3** — cleanup discard hook (CR 514.1) | **new `Command` ⇒ PROTOCOL bump** | The first finding that needs new wire surface. Smallest possible pilot for the pending-decision pattern: one player, one list, one moment |
| **PB-DP8** | **DP-6 / OOS-M11-4** — trigger-target hook (CR 603.3d) | **new `Command` + pending state ⇒ PROTOCOL + HASH bump** | The big one: 84 `Complete` defs. Should follow PB-DP7 so the pending-decision shape is already proven |
| **PB-DP9** | **DP-7 / DP-8 / DP-9** — search, scry, surveil hooks | **new `Command`(s)** | 98 `Complete` defs between them. Scry and surveil are the two whose auto-choice actively inverts the printed mechanic |
| **PB-DP10** | **gate widening** — extend `effect_choose_gate.rs` beyond its three variants, or add a `Completeness` marker distinguishing "correct" from "correct only if the engine's guess matches the player's" | none (test-only) | Without it, the 277-def figure in §3.1 grows silently with every new card. This is the invariant-level fix; the rest are instances |

**Sequencing note.** PB-DP1..PB-DP6 need no wire change at all and could ship inside the
existing paused-queue cadence. PB-DP7..PB-DP9 are a coherent block that should be planned
together, because they all need the same missing machinery: a pending-decision that actually
*gates* progress. The engine already has that machinery seven times over
(`pending_commander_zone_choices`, `pending_zone_changes`, the three payment vectors,
`DredgeChoiceRequired`, `MiracleRevealChoiceRequired`) — but §4.4 and §4.11 show that only
`pending_zone_changes` genuinely blocks. **The generalisable design work is "make a pending
decision actually block", not "add another pending vector".**

### 8.1 Seeds filed by shipped PB-DP work

Durable inventory for seeds this suite discovers. `memory/primitive-wip.md` is rewritten
wholesale by the next `/implement-primitive` run, so a seed recorded only there is lost —
seeds land **here**, in the suite's own binding spec. Same role §1c plays for the RS queue in
`memory/primitives/rider-seed-triage-2026-07-19.md`.

| seed | finding | class | status |
|---|---|---|---|
| **OOS-DP1-1** | **Echo / cumulative upkeep / recover reassign priority to the active player out of band.** `rules/engine.rs` `handle_pay_echo` / `handle_pay_cumulative_upkeep` / `handle_pay_recover` write `priority_holder = Some(active_player)` at *resolution* time, when no player holds priority — so CR 117.3c's antecedent is false and PB-DP1 correctly left the behaviour alone (comment-only fix). The write is a bodge standing in for the payment pause **DP-11** says was never implemented. Correct fix is the pause itself, owned by **PB-DP4**. **CLOSED by PB-DP4 (`scutemob-152`), by deletion**: all three `priority_holder = Some(active_player)` / `players_passed = OrdSet::new()` pairs are gone. Why they were safe to delete rather than correct: `resolve_top_of_stack` already clears the pass set and grants priority to the active player at the end of *every* resolution, so for echo and cumulative upkeep (controller **is** the active player) the writes were identity writes — but for **recover** the controller can be non-active, and the write was actively **yanking priority away** from whoever legitimately held it. Answering a resolution-time payment is neither a CR 117.3c action nor a CR 117.4 action-between-passes, so neither write had a rule behind it. Deleting them is also load-bearing for DP-11's design: a sweep that called a handler which reset the pass round mid-`handle_all_passed` would corrupt the very round it runs in. | correctness, deferred | filed by PB-DP1 (`scutemob-149`); **CLOSED by PB-DP4 (`scutemob-152`)** |
| **OOS-DP1-2** | **Residual missing entry priority guards, and the separate CR 606.3 sorcery-timing gap.** PB-DP1's fix cycle added an entry priority guard to `handle_turn_face_up`, `handle_activate_loyalty_ability` and `handle_level_up_class`. Still unguarded: `handle_activate_craft` (AP-gated by construction, so lower severity) and `handle_bring_companion` (its `:941` sorcery-speed gate does not imply the actor held priority; it resets `players_passed` unconditionally, so an active player who already passed can restart the pass round). **Separately**, loyalty and level-up still lack the CR 606.3 / 716.2a "only during their own main phase, stack empty" check — that is a timing gap, not a priority gap, and belongs to **DP-21**. Two PB-DP1 probes (`test_dp1_loyalty_activation_grants_actor_priority`, `test_dp1_level_up_class_grants_actor_priority`) exercise p2 acting on p1's turn and will need rewriting when that gate lands. | correctness, partial | partially closed by PB-DP1 (3 of 5 guards) |
| **OOS-DP1-3** | **Stale pre-renumber CR citations survive corpus-wide.** `116.3a/b/c/d` is the pre-renumber name of today's `117.3a-d` / `117.4`. PB-DP1 fixed every in-engine occurrence; ~60 golden-script `"note"` fields, one `cr_sections_tested` array, `docs/mtg-engine-milestone-reviews.md:326-327` and seven `memory/abilities/*.md` records still carry it. Cosmetic — batch into a doc pass, not a PB. | cosmetic | filed by PB-DP1 (`scutemob-149`) |
| **OOS-DP1-4** | **Mana abilities do not reset `players_passed` — a real CR 117.4 deviation, deliberately preserved.** CR 117.4 ends a pass round only when players pass "without taking any actions in between," and activating a mana ability *is* an action, so the non-reset is a genuine deviation rather than the settled behaviour CR 117.3b's parenthetical was cited for. Preserved under PB-DP1's explicit PRESERVE directive and now **pinned in two places**: probe `test_dp1_mana_ability_does_not_reset_players_passed` and golden script `stack/066_krosan_grip_split_second_blocks_counterspell.json`, whose all-pass round completes only because of it. Whoever closes this must edit both. | correctness, deviation | filed by PB-DP1 (`scutemob-149`) fix cycle |
| **OOS-DP2-1** | **`handle_keep_hand` never verifies that `cards_to_bottom` entries are in the player's hand.** It checks only the *count* (`rules/commander.rs:877-885`) and then moves each id from wherever it happens to be. A malformed or hostile `KeepHand` can bottom a card from the battlefield, from a graveyard, or **from another player's hand**. Needs an `obj.zone == ZoneId::Hand(player)` guard per entry, plus a duplicate-id check. | correctness, validation gap | filed by PB-DP2 (`scutemob-150`) |
| **OOS-DP2-2** | **Starting hand size is hard-coded to 7.** `handle_take_mulligan` draws `for _ in 0..7`; CR 103.5 says "equal to their **starting hand size**", and CR 103.5a (Vanguard) plus any starting-hand-size-modifying effect can change it. No `starting_hand_size` exists on `PlayerState`. Adding one is a **HASH bump**, so it is its own PB. | correctness, deferred (wire) | filed by PB-DP2 (`scutemob-150`) |
| **OOS-DP2-3** | **All engine shuffles are predictable from public state.** `timestamp_counter` is a hashed, replayable field, so any client that can compute the state can compute every future shuffle — including its opponents' libraries. Pre-existing and engine-wide (4 sites), **not** introduced by PB-DP2; the deck order is already deterministic from `build_initial_state`. Bears on Architecture Invariant 7 and M10's hidden-information story: a networked build needs a server-held secret seed. | security / hidden-info, M10-gated | filed by PB-DP2 (`scutemob-150`) |
| **OOS-DP2-4** | **The seeded-shuffle idiom is copy-pasted at 4 sites** (`effects/mod.rs:3048`, `:3147`, `:8701`, and now `rules/commander.rs`). A `GameState::shuffle_library_seeded(&mut self, player)` on `state/mod.rs` would dedupe it. Deliberately not done in PB-DP2 (plan §4.4): the extraction touches three proven call sites with different error-handling contracts — `move_zone_all_then_shuffle` uses `expect_move_object_to_zone`, which swallows move errors, the opposite of what the mulligan handler needs per MR-M9-12. Batch into a cleanup pass. **PB-DP2 review addendum (finding 3):** `StdRng` is not algorithm-stable across `rand` major versions — a future `rand` bump would silently re-permute every seeded shuffle with no fingerprint gate catching it (`PROTOCOL_SCHEMA_FINGERPRINT` digests the type closure, not shuffle output), and PB-DP2 widens the blast radius from a single effect to the opening library order of every game. The eventual dedup helper should pin the PRNG (`rand_chacha::ChaCha8Rng`, or an in-tree Fisher-Yates) rather than `StdRng` (SR-8/SR-9b). | cosmetic / refactor | filed by PB-DP2 (`scutemob-150`); addendum filed by PB-DP2 review |
| **OOS-DP2-5** | **`RandomBot`/`HeuristicBot` send `KeepHand { cards_to_bottom: vec![] }` unconditionally** (`crates/simulator/src/random_bot.rs:240-242`), which `handle_keep_hand` rejects after a 2nd mulligan. Unreachable today — the whole mulligan path is gated off in the simulator (`legal_actions.rs:186` needs `turn_number == 0`, but `GameStateBuilder` defaults it to 1; see the in-source statement at `crates/simulator/src/local_game.rs:569-574`) — but it goes live the moment M11-local Session 2 sets `turn_number = 0`. | correctness, latent; M11-local S2 owns | filed by PB-DP2 (`scutemob-150`) |
| **OOS-DP2-6** | **The engine defers CR 103.5's bottoming from take-time to keep-time.** CR 103.5 puts shuffle → draw → bottom N all inside "take a mulligan"; the engine does `TakeMulligan` = shuffle + draw 7 and `KeepHand` = bottom `mulligan_count - 1` (the 103.5c free-first adjustment). Behaviourally equivalent — final opening-hand size is identical, and the next mulligan reshuffles the whole library anyway, so no observable difference survives — but it is a documented divergence from the CR sentence and would matter if any future effect ever observed the library between a mulligan and a keep. Record-only; no action recommended (moving the step would need a `cards_to_bottom` field on `Command::TakeMulligan` ⇒ PROTOCOL bump). | documentation / known divergence | filed by PB-DP2 (`scutemob-150`) |
| **OOS-DP2-7** | **Two more phantom `LibraryShuffled` emitters, plus a top/bottom inversion.** `ReplacementModification::ShuffleIntoOwnerLibrary` (CR 701.20) emits `GameEvent::LibraryShuffled` at `rules/replacement.rs:854` and `:965` without ever calling `Zone::shuffle` — the identical Architecture-Invariant-4 defect PB-DP2 fixed in `handle_take_mulligan`, still live at two sites, and reachable in ordinary play (unlike the mulligan). The comment at `:849` claims "Redirect to library AND shuffle the library"; only the redirect happens, and it lands the card on the library **top** (`push_back`), so a Darksteel Colossus that dies is redrawn next turn. `crates/card-defs/src/defs/darksteel_colossus.rs` is the only def using the variant; it is `known_wrong`, but its `completeness` note at `:56-59` asserts the replacement "itself is correct", which is false at the match arm — that note (and its stale header comment at `:9`) needs fixing alongside the code. The only test (`crates/engine/tests/core/card_def_fixes.rs:1115-1121`) asserts the event's presence and never the library's contents. Fix needs a seeded `Zone::shuffle` at both sites plus a position assertion in the test. | correctness, live-wrong (gated by `known_wrong`) | filed by PB-DP2 (`scutemob-150`) review |
| **OOS-DP2-8** | **CR 103.5's mulligan cap ("until their opening hand would be zero cards") is unenforced.** `handle_take_mulligan` (`rules/commander.rs:802-877`) has no cap; `handle_keep_hand` computes `required_bottom = mulligan_count.saturating_sub(1)` (`:901`), so past 8 mulligans `required_bottom` exceeds hand size and `KeepHand` becomes unsatisfiable from the hand (long before that, the draw loop at `:864-868` silently short-draws on an exhausted library). Pre-existing, not introduced by PB-DP2. Needs a cap in `handle_take_mulligan`; no wire change. | correctness, latent | filed by PB-DP2 (`scutemob-150`) review |
| **OOS-DP3-1** | **Escalate derives a contiguous mode set `0..=count` from an empty `modes_chosen`.** CR 702.120a permits *any* set of `count + 1` distinct modes; `rules/resolution.rs:321-334` always takes the first `count + 1` in printed order, so a Blessed Alliance escalated once can never be "gain 4 life + opponent sacrifices an attacker" (modes 0 and 2). PB-DP3 validates the derived *count* against `min_modes`/`max_modes` but deliberately leaves the *identities* alone — that is escalate's semantics, not DP-4's. **Not reachable from a legal deck**: both escalate defs (`blessed_alliance.rs:102`, `collective_resistance.rs:99`) are `Completeness::partial`, so `validate_deck` blocks them; no `Complete` card is live-wrong through this path. Fix = require explicit `modes_chosen` on escalate casts and cross-check `modes_chosen.len() == count + 1`. Blast radius: 9 tests in `mechanics_e_l/escalate.rs`, `primitives/pb_ac4_per_mode_targeting.rs:834` (whose error-message assertion depends on the current path), 2 `partial` defs, golden script `stack/148`. No wire change. | correctness / agency loss, gated by `partial` | filed by PB-DP3 (`scutemob-151`) |
| **OOS-DP3-2** | **A modal *Spell* with `min_modes: 0` cast with zero modes announced is unrepresentable, and PB-DP3 hard-rejects it.** CR 700.2a permits announcing zero modes on "choose up to N"; the engine cannot express it because `rules/resolution.rs:335-338` maps an empty `modes_chosen` on a Spell stack object to `vec![0]`, and `StackObject.modes_chosen` is a bare `Vec<usize>` (`crates/card-types/src/state/stack.rs:413`) with no discriminator to separate "controller chose zero" from a free-cast that never announced anything. Latent — no such card exists (the corpus's only `min_modes: 0` object is the *triggered* `hullbreaker_horror.rs:35-59`), and the **activated** path handles the shape correctly already (`rules/abilities.rs` leaves the base effect, resolving no mode). The Spell-rejects / Activated-accepts asymmetry is deliberate and documented at both code sites. Fix needs a discriminator (`Option<Vec<usize>>` on `StackObject`, or a `modes_announced` flag) ⇒ **HASH bump**, so it is its own PB. | correctness, deferred (wire) | filed by PB-DP3 (`scutemob-151`) |
| **OOS-DP3-3** | **Four free-cast producers bypass mode announcement entirely.** `rules/copy.rs:386` (cascade), `copy.rs:614` (discover), `rules/resolution.rs:5167` (cipher copy) and `resolution.rs:5837` (suspend free-cast) build `StackObjectKind::Spell` objects with `modes_chosen: vec![]` — the last two via `StackObject::trigger_default` (`stack.rs:517-555`), which zero-fills the field — without ever calling `handle_cast_spell`. PB-DP3's cast-time guard therefore cannot reach them, and they still auto-select mode 0 via `resolution.rs:335-338`. **This is why that fallback must stay live**; deleting it as apparent dead code would make every suspended or ciphered modal spell resolve nothing. Note the correction: PB-DP3's plan originally named `rules/engine.rs:2112/2176/2686/2853` here, but those build **RingAbility / RoomAbility / LoyaltyAbility / ClassLevelAbility** stack objects and cannot reach the arm at all — the list above is the verified one. Already covered by **DP-20**; cross-referenced from its §5 row. | correctness (DP-20 scope) | filed by PB-DP3 (`scutemob-151`), site list corrected by its review |
| **OOS-DP3-4** | **Modal *triggered* abilities auto-select mode 0 at queue time, and the "choose up to one" branch is dead code.** `rules/abilities.rs:8419-8429` sets `stack_obj.modes_chosen = vec![0]` for every modal trigger with at least one mode. Its `if min_modes == 0 { vec![0] } else { vec![0] }` has **two identical branches** — the "choose up to one" case was written and then not honoured, so `hullbreaker_horror` (CR 700.2b) always bounces something and can never decline. CR 700.2b also says "If no mode is chosen, the ability is removed from the stack", which the engine never does. Adjacent to **DP-6** (trigger-target auto-selection); bundle with **PB-DP8**. | correctness / agency loss | filed by PB-DP3 (`scutemob-151`) |
| **OOS-DP3-5** | **The cast-time `ModeSelection` lookup is neither face-aware nor half-aware.** `rules/casting.rs:3495-3506` reads `def.abilities` directly rather than the active face (the PB-OS4b/PB-RS4 contract) and ignores `def.adventure_face`, while `rules/resolution.rs:246-264` *does* consult the adventure face for modes. **Second sub-case found in review:** the lookup also ignores `casting_with_aftermath`, even though the `requirements` lookup ~120 lines below does branch on it — so an aftermath cast of a card with a modal front half would now be **hard-rejected** rather than silently mis-validated. Latent both ways: the intersection of the 41 `min_modes` defs with the adventure/aftermath defs is **empty** (verified by set intersection, twice). Same root-cause class as OOS-OS4-2 / OOS-RS-3. | correctness, latent | filed by PB-DP3 (`scutemob-151`), aftermath sub-case added by its review |
| **OOS-DP3-6** | **An escalate `count` larger than `modes.len() - 1` is silently clamped, not rejected.** `rules/casting.rs:3538` and `rules/resolution.rs:332-333` both `.min(modes.len())`, so paying escalate ×5 on a 3-mode spell costs 5 extra payments and yields 3 modes. CR 702.120a's additional cost is "for each mode you choose beyond the first", and a player cannot choose 6 modes of a 3-mode spell — the announcement is illegal, not clamp-able. The current behaviour is pinned as intended by `mechanics_e_l/escalate.rs::test_escalate_modes_exceed_available_clamped`, so closing this must edit that test. Latent for `Complete` cards (both escalate defs are `partial`). Fix belongs with OOS-DP3-1's escalate PB. | correctness, latent | filed by PB-DP3 (`scutemob-151`) review |
| **OOS-DP3-7** | **Replay-harness cast-action mode parity (DP-24 class).** Only `cast_spell`, `cast_spell_modal`, `cast_spell_entwine` and `cast_spell_escalate` can announce modes; ~28 alt-cost cast arms in `crates/engine/src/testing/replay_harness.rs` (flashback, evoke, bestow, miracle, escape, foretell, plot, warp, pitch, overload, retrace, jump-start, aftermath, prototype, dash, blitz, impending, emerge, spectacle, surge, cleave, mutate, morph, commander free-cast, …) hard-code `modes_chosen: vec![]`. Before PB-DP3 that silently discarded the script's `modes` field; **after** it, those actions can never cast a modal card at all — the cast is hard-rejected. Latent: no corpus card is both modal and alt-cost-castable. A note at the `cast_spell` arm records the asymmetry. | test-infrastructure gap, latent | filed by PB-DP3 (`scutemob-151`) review |
| **OOS-DP3-8** | **The `entwine_paid` arm bypasses all mode validation.** `rules/casting.rs:3510-3516` passes `modes_chosen` through unsorted and unchecked when entwine is paid, so out-of-range or duplicate indices reach the `StackObject`. Harmless today — resolution ignores `modes_chosen` entirely under entwine (`resolution.rs:313-316`, CR 702.42a expands to all modes) — but it is now the **only** unvalidated arm of the post-PB-DP3 match, and `rules/modal.rs::test_modal_entwine_overrides_modes_chosen` pins the pass-through. Deliberately left alone (plan §11 risk 8): entwine's own keyword validation lives at `casting.rs:2845+` and is not DP-4's business. | cosmetic / hardening | filed by PB-DP3 (`scutemob-151`) review |
| **OOS-DP3-9** | **`mtg-fuzzer` aborts with a stack overflow, and long games trip `stack_consistency`.** `cargo run --release --bin mtg-fuzzer -- --games 15 --seed 1` dies with `fatal runtime error: stack overflow, aborting` (SIGABRT / exit 134). **Pre-existing, not a PB-DP3 regression** — reproduced identically on `main` (`7e0596a5`) in a scratch clone at seeds 1 and 7, and confirmed again on this branch. `--games 5` completes, so it is game-count- or game-length-dependent rather than a startup fault. That same 5-game run emits a flood of `[stack_consistency] Object ObjectId(NNNN) in stack_objects but not in Stack zone (turn 191)` invariant violations, i.e. `state.stack_objects` and the `Stack` zone have diverged. Both symptoms sit in the 150-200+ turn regime that **OOS-M11-3** (fuzzer not run-to-run deterministic) already flags, and a `stack_objects`/zone divergence is a plausible shared root cause for all three — worth investigating together. **The crash reports it writes are not replayable**: each `crash-reports/crash_N.json` records `"total_commands": 10868` alongside an **empty** `"command_history": []`, so the one artifact meant to make a violation reproducible carries no reproduction — and the file's own `Replay violations with: mtg-fuzzer --replay <SEED>` hint leans on the seed determinism that OOS-M11-3 says is absent at exactly these turn counts. That is an `event-log-diagnosability` audit concern as much as a fuzzer one. Minor hygiene rider: `crash-reports/` is not in `.gitignore`, so a fuzzer run leaves untracked artifacts in a worker's tree. **Consequence for this suite**: the fuzzer binary is not currently usable as a smoke test for simulator-side changes (PB-DP3's bot-side edit was covered by the simulator's own unit tests instead). Bears on M10a and Tier 1 hashing. | correctness / test-infrastructure, pre-existing | filed by PB-DP3 (`scutemob-151`) `/review` |
| **OOS-DP4-1** | **A hybrid / Phyrexian / X attack tax is unpayable and is now hard-rejected.** `Command::DeclareAttackers` has no `hybrid_choices` / `phyrexian_life_payments` field, so the engine cannot ask which half of a `{2/W}` tax the player pays. Before PB-DP4 the pips were **silently dropped** by `combat.rs:221-227`'s field sum (a `{2/W}` tax contributed 0 and the attack was free — the OOS-RS-2 class); PB-DP4 rejects the declaration instead, and the fix cycle rescoped the rejection so it fires only when a declared attacker actually targets the unpayably-taxed defender (attacking an untaxed opponent, or a planeswalker, is unaffected). Latent: both corpus restriction defs (`propaganda.rs`, `ghostly_prison.rs`) are pure `{2}` generic. Fix needs the two `DeclareAttackers` fields PB-RS2 added to `ActivateAbility`/`TapForMana` ⇒ **PROTOCOL bump**, so it is its own PB. | correctness, latent (wire) | filed by PB-DP4 (`scutemob-152`) |
| **OOS-DP4-2** | **CR 508.1i's mana-ability window between attack-cost determination and payment does not exist.** CR 508.1h locks in the total, 508.1i gives the active player "a chance to activate mana abilities", 508.1j then pays. The engine determines *and* pays inside one `DeclareAttackers`, so the tax must already be floating — a player with untapped lands and an empty pool cannot attack past a Propaganda. Pre-existing (PB-DP4 made the payment real but did not change the window). Fix needs a two-phase declaration ⇒ a new `Command` ⇒ **PROTOCOL bump**. The same gap exists for every "as it attacks" cost. | correctness, deviation (wire) | filed by PB-DP4 (`scutemob-152`) |
| **OOS-DP4-3** | **Goad's *directional* requirement has no CR 508.1d cost carve-out.** PB-DP4 gave both must-attack "able" tests (the `combat.rs` goad block and the `MustAttackEachCombat` block) the CR 508.1d "not required to pay" carve-out via `has_uncosted_attack_target`. The separate goad check "must attack a player other than the goading player if able" (`combat.rs:336-374`) still computes `has_non_goading_target` from opponent liveness only, so a goaded creature can be forced onto a *taxed* non-goading opponent when the untaxed goading player was the free option. Same root cause as OOS-RS3-4, narrower reach. No wire change. | correctness, narrow | filed by PB-DP4 (`scutemob-152`) |
| **OOS-DP4-4** | **The replay harness cannot script an echo or cumulative-upkeep payment.** `testing/replay_harness.rs` implements `pay_recover` (`:906-923`) and **no** `pay_echo` / `pay_cumulative_upkeep` action, so a golden script can never exercise the CR 702.30a / 702.24a *payment* branch — only PB-DP4's forced *decline* at the boundary is script-reachable. Latent today (`stack/151` is retired, `stack/152` never reaches an upkeep). DP-24 class. Deliberately not added speculatively: a harness action with no script is dead code SR-9c would flag. | test-infrastructure gap, latent | filed by PB-DP4 (`scutemob-152`) |
| **OOS-DP4-5** | **A forced decline is indistinguishable from a player's decline in the event stream.** PB-DP4's boundary sweep calls the same `handle_pay_*(pay: false)` path a real `Command::PayEcho { pay: false }` takes, so the emitted `CreatureDied` / `RecoverDeclined` carry no marker saying "the engine declined this for you because you never answered." A playtester cannot tell a rules outcome from an engine default. Fixing it inside the engine needs an event field ⇒ **HASH/PROTOCOL bump**; the cheap fix is §9 rec 8's "engine chose this for you" annotation, derived client-side in M11-local Session 7 from the absence of a preceding `Pay*` command. | diagnosability / agency visibility | filed by PB-DP4 (`scutemob-152`) |
| **OOS-DP4-6** | **`ManaPool`'s restricted-mana docs cite CR 106.12, which is the wrong rule.** Live CR 106.12 is *"To 'tap [a permanent] for mana' is to activate a mana ability … that includes the {T} symbol"*. The restricted-mana rule is **CR 106.6** ("Some spells or abilities that produce mana restrict how that mana can be spent"). Stale cites survive at `crates/card-types/src/state/player.rs:20`, `:46`, `:209` and in this audit's §4.9 `ChooseCreatureType` row. PB-DP4 fixed only `:147` and `:181` (the two docs governing the API DP-10 calls). Same class as PB-DP2's stale `103.4b` and OOS-DP1-3's pre-renumber `116.3a`. Batch into a doc pass. | cosmetic / stale cite | filed by PB-DP4 (`scutemob-152`) |
| **OOS-DP4-7** | **Three near-duplicate mana-cost arithmetic helpers.** `rules/engine.rs` `multiply_mana_cost` (multiplies every field including hybrid/Phyrexian/X — correct for cumulative upkeep), `rules/combat.rs`'s new `add_mana_cost` (rejects those fields — correct for an attack tax), and a third copy in `crates/simulator/src/legal_actions.rs` (which must mirror the engine's exactly or the SR-38 affordability gate disagrees with the engine and a bot's `PayCumulativeUpkeep { pay: true }` starts getting silently rejected). A `ManaCost` `impl Add`/`Mul` in `crates/card-types` would dedupe all three — but the semantics genuinely differ on hybrid/Phyrexian, so a naive merge would reintroduce the free-pip class (OOS-RS-2). Needs an argued API, not a rename. | cosmetic / refactor | filed by PB-DP4 (`scutemob-152`) |
| **OOS-DP4-8** | **`StubProvider` offers attack targets the player cannot pay the tax for.** `LegalAction::DeclareAttackers { eligible, targets }` (`legal_actions.rs:55-58`) lists every live opponent and opponent planeswalker with no reference to `CantAttackYouUnlessPay`, so `RandomBot::choose_attackers` routinely composes a declaration the engine rejects — and `driver.rs` answers the rejection with a silent `PassPriority`, so the bot simply loses its combat. Pre-existing (the affordability check predates PB-DP4), but PB-DP4's real debit makes it bite more often: the same floating mana no longer funds a second combat phase. SR-38 class. Fix = filter `targets`, and/or return a per-target tax so the bot can budget. Simulator-only, no wire change. | simulator move-generation gap | filed by PB-DP4 (`scutemob-152`) |
| **OOS-DP4-9** | **The echo and recover payment paths emit no `ManaCostPaid`.** PB-DP4 gave the attack tax a `GameEvent::ManaCostPaid` (Architecture Invariant 4: a pool debit is a state change and must be evented). `handle_pay_echo` and `handle_pay_recover` debit the pool and emit only `EchoPaid` / `RecoverPaid`, neither of which carries the cost; `handle_pay_cumulative_upkeep`'s `Life` arm does emit `LifeLost` but its `Mana` arm emits nothing for the debit. Deliberately not changed in PB-DP4 to keep the event-stream delta minimal for a PB whose blast radius already spans two subsystems. Wire-neutral (the variant exists); the only risk is tests that count events exactly. | diagnosability / Invariant 4 | filed by PB-DP4 (`scutemob-152`) |
| **OOS-DP4-10** | **`ActiveRestriction.controller` is captured at ETB and never recomputed for `CantAttackYouUnlessPay`.** `rules/replacement.rs:2179-2183` sets `controller` once, when the ability registers its restriction. If control of the source (e.g. Propaganda) changes hands afterward, the attack tax keeps applying against the *original* controller rather than the current one. Pre-existing since PB-18 (the affordability-only check already read this field); PB-DP4 makes it charge real mana, raising the stakes of a wrong answer from "an incorrect rejection" to "an incorrect debit, or an incorrectly-skipped debit". Fix needs either a per-check recompute of current control at declare-attackers time, or a broader audit of whether other `ActiveRestriction` variants share the staleness. No wire change. | correctness, narrow, pre-existing | filed by PB-DP4 (`scutemob-152`) fix cycle |
| **OOS-DP4-11** | **A forced decline can strand the game on an unreachable `ChooseReplacement` wait.** If an echo or cumulative-upkeep permanent has 2+ applicable zone-change replacement effects when `force_resolve_overdue_payments` (or a direct `Pay*{pay:false}`) declines it, the `ZoneChangeAction::ChoiceRequired` arm in `handle_pay_echo` / `handle_pay_cumulative_upkeep` pushes a `PendingZoneChange` and emits `ReplacementChoiceRequired` — but no `LegalAction` exists for `Command::ChooseReplacement` in `crates/simulator`, so a bot or an M11-local seat can never answer it and the permanent sits in limbo. Exotic (needs 2+ registered zone-change replacements on the same permanent) and pre-existing on the manual `PayEcho{pay:false}` path; PB-DP4's automatic sweep reaches it without any player action, widening exposure. Fix needs `LegalAction` coverage for `ChooseReplacement` — simulator-only, no wire impact. | correctness / M11-local gap, exotic | filed by PB-DP4 (`scutemob-152`) fix cycle |
| **OOS-DP4-12** | **The DP-11 deadline can be postponed indefinitely by keeping the stack non-empty.** `force_resolve_overdue_payments` fires only in `handle_all_passed`'s stack-**empty** branch. If any player (not necessarily the one who owes) casts a spell or activates a non-mana ability before the round would otherwise end, the stack is non-empty and the deadline does not fire that round — repeatably, for as long as something keeps landing on the stack. Meanwhile the permanent (or recover card) survives in its pre-consequence state: tappable, sacrificable to another cost, targetable, millable, re-triggerable. Bounded only by the postponing player's resources, not by a fixed number of rounds. The eventual outcome is still CR-correct (CR 118.12a); only the *timing* is looser than "one extra round" suggests. **No fix prescribed** — this is the shape of the CR 608.2d deviation already accepted as the smallest available under the no-new-`Command` constraint; a fix needs either a new `Command` (a stack-empty check independent of priority) or a more aggressive deadline design. Documented precisely in `engine.rs`, `resolution.rs` and `state/mod.rs`. | design-deviation, documented | filed by PB-DP4 (`scutemob-152`) fix cycle |
| **OOS-DP4-13** | **Three PB-DP4 test-hardening gaps, from the closing `/review`.** (a) **No parity test binds the simulator's `multiply_mana_cost` copy to the engine's.** `legal_actions.rs`'s copy must mirror `rules/engine.rs`'s private original exactly or the SR-38 contract breaks *silently*: the provider offers a `PayCumulativeUpkeep { pay: true }` the engine then rejects, and `driver.rs` substitutes a `PassPriority`, so the bot just loses the permanent with no error. A doc comment says so; nothing enforces it. This is the executable half of **OOS-DP4-7**. (b) **CR 508.1j's "partial payments are not allowed" is unpinned** — the only multi-source attack-tax test is exactly funded; there is no underfunded multi-attacker probe asserting `Err` **and** an untouched pool. Structurally safe today (affordability precedes every mutation and the debit is one atomic `pay_cost`), but the safety is a property of the current code shape, not a tested invariant. (c) **Two edge cases uncovered**: a zero-cost attack tax (does the `ManaCost::default()` skip emit a spurious `ManaCostPaid { cost: 0 }`?) and a pending payment owed by a player who has already lost (verified by inspection to drain safely — `turn_order` is never pruned on loss — but not pinned by a test). All three are test debt, not defects; no wire change. | test-coverage | filed by PB-DP4 (`scutemob-152`) closing `/review` |
| **OOS-DP5-1** | **`Command::OrderReplacements` has no `LegalAction`.** `crates/simulator/src/legal_actions.rs` never offers it — `crates/simulator` and `tools/` contain **zero** occurrences of `OrderReplacements` — so no bot and no M11-local seat can ever answer a CR 616.1 prompt, for a **draw or a zone change**. The pre-existing `pending_zone_changes` path has the identical hole, so PB-DP5 did not create it; it made it matter, because there is now a second prompt kind that expects an answer. Same class as PB-DP4's §9 recommendation and **OOS-DP4-11** (`ChooseReplacement`). Simulator-only, no wire change. **Fold in here**: a `PendingDraw` is selected FIFO (`.position(|p| p.player == player)`), so a player holding two outstanding pending draws cannot say *which* one they are answering — an answer aimed at the newer entry is silently applied to the older. Card-neutral today and harmless while both entries are draws of the same shape, but a per-entry discriminator belongs in whatever surface finally exposes the choice. | agency / move-generation gap | filed by PB-DP5 (`scutemob-153`) |
| **OOS-DP5-2** | **No deadline for an unanswered `PendingDraw` / `PendingZoneChange`.** PB-DP4's `force_resolve_overdue_payments` has no CR 616.1 analogue. Deliberately **not** built in PB-DP5: with no `LegalAction` (OOS-DP5-1) a sweep would fire in ~100% of automated games and become a new "the engine chose for you" site — precisely the thing this PB exists to remove — for zero benefit today, since DP-5 is unreachable from a legal deck. Should ship **together with** OOS-DP5-1 as one PB. **Fold in here**: `pending_draws` entries are never cleaned up either — not at end of turn, not on player loss, not when the replacement's source leaves the battlefield. Not exploitable (every submitted id must still be *currently* applicable, and `validate_player_active` blocks a lost or conceded sender), but a stale entry feeds the state and loop-detection fingerprints forever and hands the drawing player unbounded timing control over an owed draw. A deadline sweep is the natural place to reap them. | correctness, deferred (design) | filed by PB-DP5 (`scutemob-153`) |
| **OOS-DP5-3** | **The resume never re-offers dredge.** `resolve_pending_draw` performs the deferred draw and the `remaining` sequence with `offer_dredge: false`, so CR 702.52a is not offered on draws 2..N of a resumed sequence. Deliberate: re-offering would restart a CR 616.1 chain the player has already begun and would open a second pause with nowhere to record it. Presently unobservable — the intersection of "has a dredge card" and "has 2+ `WouldDraw` replacements" is empty, because there are **no** `WouldDraw` cards at all. | correctness, stated deviation | filed by PB-DP5 (`scutemob-153`) |
| **OOS-DP5-4** | **`PlayerState::has_drawn_for_turn` is write-only dead state, and incoherently written.** No engine logic ever reads it. It is written by `turn_actions::draw_card` — which also serves the monarch end-step draw and Ravenous, neither of which is "the draw for the turn" — and by `draw_card_skipping_dredge`, so an *effect* draw routed through a dredge decline sets it while the direct effect path does not. PB-DP5 preserved the existing writes byte-for-byte behind a `PendingDraw.sets_has_drawn_for_turn` flag rather than unifying them, because the field **is** hashed: deleting or normalising it is a HASH bump and cannot be a drive-by. | cosmetic / dead state (wire) | filed by PB-DP5 (`scutemob-153`) |
| **OOS-DP5-5** | **A deferred draw does not stop the rest of the effect.** Only the *draw sequence* stops (CR 614.11a). "Draw three, then discard three" runs its second half against a hand that does not yet hold the drawn cards — a CR 614.11a / 121.1 timing deviation. Strictly better than the status quo (before PB-DP5 the draws were destroyed outright), and the alternative — suspending mid-resolution — needs a suspendable effect resolver, i.e. exactly the pending-decision machinery §8's sequencing note calls for in PB-DP7..DP9. | correctness, stated deviation | filed by PB-DP5 (`scutemob-153`) |
| **OOS-DP5-6** | **`WouldDraw` honours only `SkipDraw`.** Every other `ReplacementModification` on a draw is a silent no-op that does not even emit `ReplacementEffectApplied`. This is why two draw replacements can never differ in *outcome*, and why PB-DP5's order-honoured tests had to discriminate on the event stream rather than on game state. At least three `inert` defs are blocked on it: `laboratory_maniac.rs`, `teferis_ageless_insight.rs` and `out_of_the_tombs.rs` (which names the blocker verbatim in its completeness note). Minimum widening: `DrawAdditionalCards(u32)` (Alhammarret's Archive, Teferi's Ageless Insight) + a skip-and-redirect variant (Notion Thief) + a replace-with-effect variant (Laboratory Maniac). **This is the PB that gives DP-5's machinery any card yield at all** — own PB, and the natural next step after PB-DP5. | DSL gap / card yield | filed by PB-DP5 (`scutemob-153`) |
| **OOS-DP5-7** | **`Command::ChooseDredge` has NO pending-state gate — a live, reachable exploit.** `rules/engine.rs` validates only that the player exists; `handle_choose_dredge` validates the *card* (in graveyard, has `Dredge(n)`, library ≥ n) but **never that a draw is pending**. So any player, at any time, can send `ChooseDredge { card: None }` and take a **free extra card** (`draw_card_skipping_dredge` then draws unconditionally), or send `ChooseDredge { card: Some(x) }` and dredge at will. Exactly DP-5's trust-boundary class, but **live and reachable today** — dredge defs exist (`golgari_grave_troll.rs`) and the command is script-reachable. The fix reuses PB-DP5's machinery almost verbatim: give the `DredgeAvailable` pause its own `PendingDraw`-style entry and require-and-consume it in `handle_choose_dredge`. Existing dredge tests (`tests/mechanics_a_d/dredge.rs`) and golden script `replacement/014` all reach `DredgeChoiceRequired` first, so they would stay green. **Deliberately not folded into PB-DP5** (implement-phase default-to-defer), and confirmed still true on that branch. **Arguably a higher-severity finding than DP-5 itself** — it is the only item in this seed list that is wrong in a game you could play today. | correctness / trust boundary, **live** | filed by PB-DP5 (`scutemob-153`) |
| **OOS-DP5-8** | **The `AutoApply` draw arm applies an effect and emits nothing.** With exactly one applicable non-`SkipDraw` `WouldDraw` replacement, the effect is neither applied nor recorded — no `ReplacementEffectApplied`, no `already_applied` entry, no event at all. A player watching the log cannot tell the replacement exists. Same diagnosability class as **OOS-DP4-5**. Fold into the OOS-DP5-6 widening PB. | diagnosability | filed by PB-DP5 (`scutemob-153`) |
| **OOS-DP5-9** | **`already_applied` is not threaded out of the CR 616.1f re-check, so a re-offered id can be applied twice.** `perform_one_draw` records into `PendingDraw` the `already_applied` set it was *called* with, not any id that `check_would_draw_replacement`'s own internal re-check auto-applied on the way to the `NeedsChoice`. For an applicable set of `{S: a non-SkipDraw self-replacement, X, Y}` — CR 616.1a makes `determine_action` return `AutoApply(S)`, and the re-check then yields `NeedsChoice` on `{X, Y}` — the pushed entry's `already_applied` is **empty**, so on resume `find_applicable` re-offers `S` and a client can submit an id that was never in the offered `choices`, applying `S` a second time (CR 614.5 says each replacement applies once). **Inherited, not introduced**: `check_zone_change_replacement` has the identical gap and documents it as the registered M10 follow-up. **Unobservable today** — every non-`SkipDraw` draw modification is a game-state no-op, so "applied twice" and "applied once" are indistinguishable, which is exactly why this must be closed *with* **OOS-DP5-6**: the widening that makes draw modifications observable is the change that makes this a live double-application. Documented at the `perform_one_draw` doc comment. **Second, cosmetic item folded in:** `perform_one_draw`'s `expect_move_object_to_zone` failure path returns `DrawStepOutcome::Completed`, so a corrupted-state move failure is reported as a completed draw and a sequence resume keeps iterating; debug-assert-only, and the pre-existing `draw_one_card` had the same shape. | correctness, latent (blocked on OOS-DP5-6) | filed by PB-DP5 (`scutemob-153`) closing `/review` |

---

## 9. M11-local decision-loop extensibility assessment

Against `/home/skydude/projects/scutemob/.worktrees/scutemob-147/memory/m11-session-plan.md`
(read-only cross-worktree reference; nothing under `.worktrees/` was modified).

### 9.1 The question

The plan's §2(a) adopts a steppable driver:

```
    pub enum AdvanceOutcome { AwaitingHuman(PendingDecision), GameOver(..), Halted(..) }
    pub enum DecisionKind { Priority, Mulligan, CommanderZoneChoice,
                            DeclareAttackers, DeclareBlockers }
    pub struct HumanChoice { pub action_index: usize, pub params: ActionParams }
```

Does that generalise to trigger-time and resolution-time choices without redesign?

### 9.2 Answer: **no — and the plan's own reasoning shows why**

§2(a) justifies the design like this:

> **Sub-decisions**: the five non-`choose_action` `Bot` methods are *not* driver callbacks
> (verified: zero call sites), so there is nothing to intercept. Attackers, blockers,
> mulligan-bottoms and targets are all fields of the `Command` the seat returns. They become
> fields of `ActionParams` (§3), which is a server-side type — **no new `Command` or
> `GameEvent` variant.**

That is **exactly right for the five decisions it enumerates, and exactly wrong as a general
claim** — because it is true only of choices that are announced *at command-submission time*.
This audit's §4.6 through §4.11 are all choices that happen *inside* `process_command`, while
the engine flushes triggers or resolves a stack object. Concretely:

- `LocalGame::advance()` can only yield `AwaitingHuman` **between** `process_command` calls.
- `execute_effect_inner` returns `()` and cannot suspend (§4.9).
- `flush_pending_triggers` runs to completion inside whatever command triggered it (§4.6/4.7).
- `ActionParams` is a *command-assembly* struct. There is no `Command` for a trigger target,
  a scry split, a search pick, or a cleanup discard, so there is nothing for `ActionParams` to
  assemble.

So `DecisionKind`'s five variants are not an early cut of a longer list — they are **the
complete set of decisions reachable by this architecture**, and they will stay complete until
the *engine* grows pending-decision state for the other classes. That is a wire change, which
the plan's §7 constraint 6 explicitly forbids:

> **No new `Command` / `GameEvent` / `Effect` variant.** If one seems necessary, stop and
> flag: it is a wire change (SR-8) requiring a PROTOCOL bump, and for this milestone it
> signals a design error.

**This is not a criticism of the plan.** Constraint 6 is right for M11-local, and the audit
confirms the plan's own §1 fact 6 (optional payments are "not stalls; just silently
unavailable"). The problem is only that a closed 5-variant `DecisionKind` and a flat
`actions: Vec<LegalAction>` will *look* complete to session 5 and 7 authors, and the DTO
shapes they lock in will have to be reshaped later. The fix is cheap and belongs in the
sessions that define those shapes.

### 9.3 The engine already has the right pattern — it just doesn't block

Seven existing round-trips prove the shape generalises at the engine level:
`ReplacementChoiceRequired`/`OrderReplacements`, `CommanderZoneReturnChoiceRequired`,
`DredgeChoiceRequired`/`ChooseDredge`, `MiracleRevealChoiceRequired`/`ChooseMiracle`,
`EchoPaymentRequired`/`PayEcho`, `CumulativeUpkeepPaymentRequired`, `RecoverPaymentRequired`.
Each is: emit a `*Required` event → record a pending entry → accept an answering `Command`.

But §4.4 and §4.11 show that **only `pending_zone_changes` actually gates anything**. The
other five pending vectors are inert queues that nothing consults. So the reusable asset is
the *event + command + pending-entry triple*; the missing piece is the gate. That is why
PB-DP7 in §8 is proposed as a deliberate pilot.

### 9.4 Concrete recommendations

**Session 3** — *Action parameterization + engine target queries*

1. Make `DecisionKind` `#[non_exhaustive]` and document, in the enum's doc comment, that it
   enumerates **command-submission-time** decisions only, with a pointer to this audit for the
   trigger-time and resolution-time classes it structurally cannot reach. One comment now
   prevents a later reader assuming the surface is complete.
2. Give `PendingDecision` an explicit shape for its payload rather than only
   `actions: Vec<LegalAction>` — e.g. `payload: DecisionPayload` with an `Actions(Vec<LegalAction>)`
   variant today. A future `TriggerOrder`/`ResolutionChoice` then adds a variant instead of
   reshaping the struct that sessions 5 and 7 will have built DTOs against.
3. **Surface the decisions that are already reachable with zero wire change.** The plan's §1
   fact 6 lists them; this audit adds why they matter. `advance()` should yield `AwaitingHuman`
   for a non-empty `pending_echo_payments` / `pending_cumulative_upkeep_payments` /
   `pending_recover_payments`, and `OrderBlockers` should be offered whenever an attacker has
   ≥2 blockers. These need new `LegalAction` variants (simulator-internal, **not** a wire
   change) and are the cheapest agency the milestone can buy. Note **DP-11**: because nothing
   enforces the pending payments, an M11 game today lets every echo permanent stay for free.

   > **UPDATED by PB-DP4 (`scutemob-152`).** The `LegalAction` half is **DONE** for all three
   > payments; the enforcement gap is closed; and **the `advance()` half turned out to be
   > unnecessary**. Because the deadline makes the payment a choice inside a normal priority
   > window rather than an out-of-band pause, `LegalAction::PayEcho` / `PayCumulativeUpkeep` /
   > `PayRecover` arrive inside the **existing** `PendingDecision` and are classified
   > `DecisionKind::Priority` by `decision_kind_for` — **no new `DecisionKind` variant, no
   > `PendingDecision` reshaping, no `local_game.rs` edit at all.** Cheaper than this
   > recommendation predicted. The `OrderBlockers` half (**DP-13**) remains open and is still
   > the only decision in this list with no `LegalAction`.
4. ~~Session 3 item 5 already forbids re-deriving `hybrid_choices` / `phyrexian_life_payments`.
   Extend the same rule to `modes_chosen`: **`action_to_command_with_params` must reject an
   empty `modes_chosen` on a modal action whose `min_modes >= 1`**, because the engine will not
   (**DP-4**). Otherwise the first human to cast Cryptic Command through the play server gets
   half a spell at full price and no error.~~
   **SUPERSEDED by PB-DP3 (`scutemob-151`)** — the engine now rejects it, at cast time and
   before costs are paid, so the play server needs no compensating check. What it needs instead
   is a mode-selection **UI**: `crates/simulator/src/legal_actions.rs`'s `spell_default_modes` /
   `ability_default_modes` supply the first `min_modes` legal indices as a bot/placeholder
   default, and that placeholder is exactly what session 7 must replace with a human choice.
   Note also **OOS-DP3-7**: the ~28 alt-cost cast paths cannot announce modes at all, so a
   modal card cast via flashback/escape/etc. is unreachable through them today.

**Session 5** — *play-server REST API*

5. Make `DecisionView` a **tagged union keyed on decision kind**, not a flat
   `{seq, kind, actions}`. `POST /api/game/action` should carry the same discriminator. A
   trigger-ordering decision is a permutation, a scry decision is a partition, a cleanup
   discard is a subset — none of them is an index into an action list, and a client written
   against a flat action list will need a breaking change to accept any of them.
6. On the R4 `--dev` raw-command escape hatch the plan defers to this session: **recommend
   yes**, and note that it is also the only way a playtester can currently exercise
   `PayEcho`, `PayCumulativeUpkeep`, `PayRecover` and `OrderBlockers` (no `LegalAction`
   exists for any of them, §4.11 / DP-11 / DP-13).
   **SUPERSEDED IN PART by PB-DP4 (`scutemob-152`)** — `LegalAction::PayEcho`,
   `PayCumulativeUpkeep` and `PayRecover` now exist and reach a human seat through the normal
   priority decision, so the raw-command hatch is no longer the only way to exercise them. The
   `--dev` hatch is still recommended, and **`OrderBlockers` is now the only decision in the
   audit with no `LegalAction`**.
7. `GET /api/game/report` (session 8 item 5) should include the auto-choice annotations from
   recommendation 8 below, so a "that seemed wrong" report distinguishes an engine bug from an
   engine *choice*.

**Session 7** — *Targeting, combat and choice UIs*

8. **Ship an "engine chose this for you" annotation in the event feed.** This is the single
   highest-value item the milestone can add for this audit's findings, it needs no wire change,
   and it converts an invisible wrongness into a visible one. Minimum viable set, all
   detectable from existing `GameEvent`s plus the seat view: triggered-ability targets
   (**DP-6**), scry/surveil disposition (**DP-8/DP-9**), search picks (**DP-7**), sacrifice and
   discard picks (**DP-16**, **DP-3**), and combat damage assignment (**DP-13**).
9. `TargetPicker.svelte` is the **cast-time** picker (CR 601.2c). Name it accordingly and
   document that trigger targets (CR 603.3d) never reach it. Otherwise session 7 will close
   believing targeting is done.
10. The attacker picker should not imply the engine will enforce attack requirements
    maximally — §4.5 shows validation is per-declaration only, so a client that offers an
    under-satisfying attack set gets a rejection, not a correction. And per **DP-10**, if a
    Propaganda-class effect is out, the UI must not tell the human they paid: nothing is
    debited.
11. Session 7's acceptance is "the human can attack, block, and cast targeted/X/modal spells".
    Add: **and can see, in the feed, every choice the engine made on their behalf.**

### 9.5 What does *not* need to change

The plan's crate layout, its async boundary, `Viewer`-based redaction, the steppable-driver
decision over the channel-backed bot, and constraint 6 are all sound and this audit does not
touch them. `LocalGame` is the right home for a decision loop; it just needs to be honest
about which decisions it can currently reach.

---

## 10. Re-audit triggers

Per [methodology.md](methodology.md) "When to Re-Audit":

- Any new `Effect` variant that carries a `choices`, `optional`, `may`, or filter-selection
  field — check whether it joins §3.1 or is gated.
- Any new `Command` variant — check whether it closes a §5 finding or is another
  accepted-and-discarded field (**DP-24**).
- After **PB-DP7** or any first pending-decision-that-blocks lands — re-run §3.1's sweep and
  re-derive the 277 figure. Exclude `defs/mod.rs`, and remember the union must include the
  modal-triggered-ability row — omitting it is how the first pass of this audit undercounted by 5.
- After a `Zone` API change — **DP-2** is a top/bottom inversion of exactly the kind PB-RS1
  swept; the sweep's roster did not include the mulligan, and a future roster should be
  derived from `Zone::push_front` call sites rather than hand-listed.
- When `docs/authoring-status.md` shows a material jump in `Complete` count — the §3.1
  percentage moves with it.
