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
engine-side (`abilities.rs:7174-7500`, proposed seed OOS-M11-3), plus M11-local building a
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
| Mode announcement, **omitted** | 601.2b / 700.2a | **D** | `rules/casting.rs:3555-3559` — see **DP-4** |
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
| Modes | 700.2a | **B** | `rules/abilities.rs:386-397` — empty ⇒ `vec![0]`; same `min_modes` bypass as **DP-4** |
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
| The **shuffle** inside a mulligan | **103.4b/103.5** | **D** | `rules/commander.rs:808-848` — see **DP-2** |
| `cards_to_bottom` **placement** | **103.5** | **D** | `rules/commander.rs:886-890` — see **DP-2** |
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
| Attack requirements (goad, must-attack) | 508.1d / 701.15b | **A** | `rules/combat.rs:272-432`, with the correct requirement-yields-to-restriction carve-out at `:412-424` |
| **Attack cost** (Propaganda) | **508.1g** | **D** | `rules/combat.rs:248-263` — see **DP-10** |
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
| **`WouldDraw` multi-replacement** | **D** | `rules/turn_actions.rs:1186-1189` (and the twin at `effects/mod.rs:8553-8564`) — see **DP-5** |

### 4.11 Cleanup and other turn-structure choices (CR 514.1)

| choice | CR | class | site |
|---|---|---|---|
| Hand-size discard | **514.1** | **B** | `rules/turn_actions.rs:1280-1293` — see **DP-3** |
| "Until end of turn" expiry | 514.2 | **A** | `rules/turn_actions.rs:1369-1370`; damage clear `:1348-1350`; pools `:1375` |
| No priority in cleanup | 514.3 | **A** | `crates/engine/src/state/turn.rs:63-65` |
| Extra cleanup round | 514.3a | **A** | `rules/engine.rs:1731-1762` + the non-advance guard at `:1678-1695`, with a 100-round cap |
| Echo / cumulative upkeep / recover pay-or-sacrifice | 702.30a / 702.24a / 702.59a | **A** plumbing, **D** enforcement | see **DP-11** |
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

| id | class | finding | CR | site |
|---|---|---|---|---|
| **DP-1** | D | **Priority after casting / activating / a special action goes to the ACTIVE player, not the actor.** ~20 sites all do `priority_holder = Some(state.turn.active_player)`. CR 117.3c: "If a player has priority when they cast a spell, activate an ability, or take a special action, **that player** receives priority afterward." The comment at `casting.rs:4712` misquotes CR 601.2i as "Then the active player receives priority" (it actually reads "If the spell's controller had priority before casting it, they get priority"); `abilities.rs:1384` cites **CR 602.2e, which does not exist**. Consequence: a non-active player can never hold priority — no double-spell response, no sacrificing a creature in response to your own removal. Identical to CR only when the actor *is* the active player. | 117.3c, 601.2i | `rules/casting.rs:4712-4715`; `rules/abilities.rs:1384-1387`, `:1552`, `:1753`, `:1967`, `:2102`, `:2341`, `:2504`, `:2681`, `:2857`, `:8791`, `:9000`, `:9202`; `rules/engine.rs:757`, `:958`, `:1072`, `:1461`, `:1759`, `:1805`; `rules/combat.rs:1373` |
| **DP-2** | D | **A mulligan is a content no-op, and `cards_to_bottom` goes to the library's TOP.** `handle_take_mulligan` moves every hand card to the library — `Zone::insert` is `push_back` and `Zone::top()` is `v.last()`, so they land on top — emits a **phantom** `GameEvent::LibraryShuffled` with no permutation, then draws 7. **The same seven cards return, reversed.** Separately, `handle_keep_hand` bottoms cards with `move_object_to_zone` (top) instead of `push_front` (bottom), so the cards you bottom are the next cards you draw. This is the OOS-RS-1 top/bottom inversion class that PB-RS1 swept — the mulligan was not in its roster. Tests (`crates/engine/tests/rules/commander.rs:1400-1495`) assert hand counts and events only, never library position. | 103.4b, 103.5 | `rules/commander.rs:808-848` and `:886-890`; `crates/card-types/src/state/zone.rs:109`, `:159-164`, `:187`; `state/builder.rs:286` |
| **DP-3** | B | **Cleanup discard has no `Command` at all** and auto-picks the **highest `ObjectId`** in hand — the most recently drawn card — one at a time. Hand is `Zone::Unordered` (an `OrdSet`), `object_ids()` yields ascending, `.last()` takes the top. Madness is correctly honoured on this path (`:1301-1342`), which means the auto-picker can involuntarily fire Madness on a card the player would never have chosen. | 514.1 | `rules/turn_actions.rs:1280-1293`; `crates/card-types/src/state/zone.rs:130-135` |
| **DP-4** | D | **An empty `modes_chosen` bypasses `min_modes` entirely.** The range / duplicate / `min_modes` / `max_modes` checks live only inside the `!modes_chosen.is_empty()` branch. Cast-time target slicing then assumes `vec![0]` (`casting.rs:3645-3653`) and resolution re-derives `vec![0]` (`resolution.rs:335-341`). **Cryptic Command, Austere Command and Incendiary Command all declare `min_modes: 2, max_modes: 2` and all three are `Complete`** — cast one with no modes and it pays the full cost and resolves *one* mode, silently. The Spree path *does* hard-reject empty modes (`casting.rs:2940-2944`); the general modal path has no equivalent. | 601.2b, 700.2a | `rules/casting.rs:3506`, `:3536-3549`, `:3555-3559`, `:3645-3653`; `rules/resolution.rs:335-341`; `crates/card-defs/src/defs/cryptic_command.rs:31-32`, `austere_command.rs:27-28`, `incendiary_command.rs:37-38` |
| **DP-5** | D | **The `WouldDraw` multi-replacement prompt is unanswerable and the draw is destroyed.** `draw_card` emits `ReplacementChoiceRequired` and returns early, recording **no** pending state — there is no draw-pending field on `GameState`. But `handle_order_replacements` requires a matching `pending_zone_changes` entry and errors without one, so `Command::OrderReplacements` sent in reply is rejected and the draw can never complete. Reachable with any two `WouldDraw` replacements, including in the draw step. The existing test (`tests/rules/replacement_effects.rs:2984-3000`) asserts only that the draw was deferred. | 616.1, 614.11 | `rules/turn_actions.rs:1186-1189`; twin at `effects/mod.rs:8553-8564`; `rules/replacement.rs:163-172` |

### Tier 1 — silently wrong in most games, driven by `Complete` cards

| id | class | finding | CR | `Complete` defs | site |
|---|---|---|---|---:|---|
| **DP-6** | B | Triggered-ability targets auto-selected by first-match (**OOS-M11-3**) | 603.3d | **84** | `rules/abilities.rs:7174-7500` |
| **DP-7** | B | Library search picks the lowest `ObjectId` — every tutor fetches for you | 701.23 | **74** | `effects/mod.rs:3032` |
| **DP-8** | B | Scry sends **all** cards to the bottom; "keep on top" is unreachable | 701.22a | 16 | `effects/mod.rs:3089-3098` |
| **DP-9** | B | Surveil sends **all** cards to the graveyard; Surveil N ≡ Mill N | 701.25a | 8 | `effects/mod.rs:3123-3130` |
| **DP-10** | D | **Propaganda attack tax is checked but never charged.** The mana pool is inspected once (`combat.rs:250-253`) and never debited anywhere in the 600-line handler. Float the mana, attack free, keep the mana. Colour is also flattened to generic (`:218-227`) | 508.1g | — | `rules/combat.rs:248-263` |
| **DP-11** | D | **Echo / cumulative upkeep / recover never enforce the "otherwise, sacrifice".** `resolution.rs:2785` asserts "The game pauses until a `Command::PayEcho` is received"; **no code implements that pause.** The three `pending_*` vectors are read only inside their own handlers — never by priority, SBA or step advancement. Pass priority and the permanent is neither paid for nor sacrificed. Compounding: none of the three has a `LegalAction`, so in a bot/M11 game the command is never sent | 702.30a, 702.24a, 702.59a | — | `rules/resolution.rs:2784-2815`, `:2833-2872`, `:2891-2918`; `rules/engine.rs:598-612`, `:779-783`, `:1007-1011` |
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
| **DP-20** | B | Cascade / Discover / `PlayExiledCard` always cast; cascade free-cast also gets no targets and mode 0 | 702.85a, 701.57 | `rules/copy.rs:366-368`, `:389`, `:430`; `effects/mod.rs:3837-3848`, `:4361-4364` |
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

### OOS-M11-3 — triggered-ability targets: **CONFIRMED; reclassify B, not D**

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
| **PB-DP1** | **DP-1** — priority holder after cast / activate / special action (CR 117.3c) | **none** | ~20 mechanical sites, no new type. Highest correctness-per-line ratio in the whole audit, and it is the precondition for a human seat behaving like a player rather than a spectator |
| **PB-DP2** | **DP-2** — mulligan. Split: (a) `cards_to_bottom` → `push_front`; (b) real shuffle | (a) **none**; (b) needs a seed on `GameState` ⇒ **HASH bump** | (a) is a one-liner and closes a live-wrong path today. (b) is the OOS-M11-1 seed proper; M11-local Session 2 routes around it with a pregame `redeal`, so (b) can trail (a) |
| **PB-DP3** | **DP-4** — `min_modes` floor when `modes_chosen` is empty | **none** | 3 `Complete` cards resolve half a spell at full price. Mirror the Spree guard at `casting.rs:2940-2944` |
| **PB-DP4** | **DP-10** + **DP-11** — attack tax debit; echo/cumulative-upkeep/recover enforcement | **none** if the "otherwise" is applied at resolution rather than gated on priority | Two "the cost is checked but never collected" bugs of the same shape. Bundling them makes the shared lesson explicit |
| **PB-DP5** | **DP-5** — `WouldDraw` pending-choice state | **HASH bump** (new `GameState` field); no new `Command` if it reuses `OrderReplacements` | The engine currently asks a question it cannot accept an answer to and silently eats a draw |
| **PB-DP6** | **DP-15** — intervening-if at queue time (CR 603.4) | **none** | Already accepted as a known limitation in the defs; closing it retires a whole class of def-level caveats |
| **PB-DP7** | **DP-3** — cleanup discard hook (CR 514.1) | **new `Command` ⇒ PROTOCOL bump** | The first finding that needs new wire surface. Smallest possible pilot for the pending-decision pattern: one player, one list, one moment |
| **PB-DP8** | **DP-6 / OOS-M11-3** — trigger-target hook (CR 603.3d) | **new `Command` + pending state ⇒ PROTOCOL + HASH bump** | The big one: 84 `Complete` defs. Should follow PB-DP7 so the pending-decision shape is already proven |
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
4. Session 3 item 5 already forbids re-deriving `hybrid_choices` / `phyrexian_life_payments`.
   Extend the same rule to `modes_chosen`: **`action_to_command_with_params` must reject an
   empty `modes_chosen` on a modal action whose `min_modes >= 1`**, because the engine will not
   (**DP-4**). Otherwise the first human to cast Cryptic Command through the play server gets
   half a spell at full price and no error.

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
