# PB-SFT Re-Triage Memo — Cost::SacrificeFilteredType

**Branch**: `feat/pb-sft-re-triage-costsacrificefilteredtype-verify-scope-and-`
**Author**: worker (scutemob-8)
**Date**: 2026-04-29
**Trigger**: Re-triage discovery — verify scope before dispatching primitive batch.
**Status**: discovery only; no engine or card def files modified.

---

## Verdict line

**PROCEED — FIELD-ADDITION (Effect-side).**
The planner's title `Cost::SacrificeFilteredType` is **misnamed**: the cost surface is
already wired and shipping. The real gap is on the **Effect side** —
`Effect::SacrificePermanents` lacks a permanent-type filter, leaving ~14 card defs
holding TODOs that say "no creature filter / no nontoken filter / no creature-or-
planeswalker filter." A 1-field addition + a single resolution-site filter check
unblocks the bulk of the unblocked-shippable cards.

A secondary, narrower gap (`Cost::Sacrifice "another"` enforcement) is real but
nearly redundant — only **1 card** out of the activation-cost set unblocks cleanly
on it (Commissar Severina Raine). Optional roll-in.

---

## 1. Engine surface analysis

### 1a. The 3 callsites at `crates/engine/src/cards/card_definition.rs:3082/3131/3183`

These 3 callsites are inside **predefined token specs** (Food, Clue, Blood) — not
card def authoring sites. They each set `sacrifice_filter: None` because a Food
token's "Sacrifice this token" is `sacrifice_self: true`, not a filtered sacrifice.
There is **no field to wire up at these callsites** — they are correct as written.
The brief's hypothesis ("a `sacrifice_filter: None` field already exists at 3
callsites") is technically true but implies misuse that does not exist.

```text
food_token_spec  @ card_definition.rs:3082    sacrifice_filter: None,  // correct: source is the token
clue_token_spec  @ card_definition.rs:3131    sacrifice_filter: None,  // correct
blood_token_spec @ card_definition.rs:3183    sacrifice_filter: None,  // correct
```

### 1b. Existing `Cost::Sacrifice*` and related variants

Five separate but related surfaces exist:

| Surface | Location | Status |
|---|---|---|
| `Cost::SacrificeSelf` | `card_definition.rs:1064` | Wired everywhere; ~30+ cards use it. |
| `Cost::Sacrifice(TargetFilter)` | `card_definition.rs:1075` | Wired via `flatten_cost_into` in `replay_harness.rs:3221-3262` and `abilities.rs:675-741`. **13 cards already shipping**: vampiric_rites, blasting_station, phyrexian_altar, altar_of_dementia, goblin_bombardment, warren_soultrader, yahenni_undying_partisan, greater_good, spawning_pit, perilous_forays, siege_gang_lieutenant, carrion_feeder, ayara_first_of_locthwain, etchings_of_the_chosen, wight_of_the_reliquary. |
| `ActivationCost.sacrifice_filter: Option<SacrificeFilter>` | `state/game_object.rs:242` | Receives the lowered filter. Validated at activation time in `rules/abilities.rs:675-741` against layer-resolved characteristics (CR 613.1f), with controller / zone checks (CR 602.2). |
| `SpellAdditionalCost::Sacrifice{Creature,Land,ArtifactOrCreature,Subtype,ColorPermanent}` | `card_definition.rs:1101-1112` | Wired at `casting.rs:3199-3210`. **9 cards already shipping**: harrow, lifes_legacy, village_rites, abjure, corrupted_conviction, crop_rotation, deadly_dispute, altar_of_bone, diabolic_intent, goblin_grenade, natural_order. |
| `Effect::SacrificePermanents { player, count }` | `card_definition.rs:1503-1506` | **NO `filter` FIELD.** Resolution picks player's lowest-ObjectId permanent regardless of type. **~14 card defs use it; nearly all carry TODOs about the missing filter.** |

`SacrificeFilter` enum @ `state/game_object.rs:196-210` has 6 variants:
`Creature`, `Land`, `Artifact`, `ArtifactOrCreature`, `Subtype(SubType)`,
`CreatureOfChosenType` — all hashable (`hash.rs:2072+`) and validated against
layer-resolved characteristics in `abilities.rs:697-735`.

`Cost::Sacrifice(TargetFilter)` → `SacrificeFilter` lowering in `replay_harness.rs:3226-3251`
honors `has_chosen_subtype`, `has_subtype`, and `has_card_type` (Creature/Land/Artifact),
but **silently drops** `is_token`, `non_creature`, `colors`, `exclude_subtypes`,
`is_attacking`, and the multi-type `has_card_types: Vec<CardType>` field.

`abilities.rs:675-741` does **not enforce "another"** — a `Cost::Sacrifice(TargetFilter::Creature)`
on a creature source will accept the source itself if the activator chooses it.
Wight of the Reliquary ships with a TODO acknowledging this; this is a real but
narrow correctness gap.

### 1c. Verdict label

**FIELD-ADDITION** on `Effect::SacrificePermanents` — add
`filter: Option<TargetFilter>` and honor it at resolution.

Optional roll-in: a separate **FIELD-ADDITION** on `Cost::Sacrifice` (or on the
`SacrificeFilter` enum) for "another" enforcement — affects only ~1 currently-
unblockable card.

This is **not** NEW-VARIANT (the planner's `Cost::SacrificeFilteredType` would
duplicate `Cost::Sacrifice(TargetFilter)` which already exists) and **not** WIRE-UP-
EXISTING in the literal sense (the field literally does not exist on
`Effect::SacrificePermanents`).

---

## 2. TODO inventory

Pattern matched: card defs whose TODOs reference "SacrificePermanents has no
[type] filter", "SacrificePermanents lacks", "Cost::Sacrifice…", or "non-[X]"
sacrifice filters. `mcp__mtg-rules__lookup_card` was called for each card to
verify oracle text.

### 2a. Cost-side activation costs ("Sacrifice [filter]: …")

| # | Card | file:line | Oracle excerpt (MCP) |
|---|------|-----------|----------------------|
| 1 | Commissar Severina Raine | `crates/engine/src/cards/defs/commissar_severina_raine.rs:23` | `{2}, Sacrifice another creature: You gain 2 life and draw a card.` |
| 2 | Wight of the Reliquary | `crates/engine/src/cards/defs/wight_of_the_reliquary.rs:35` | `{T}, Sacrifice another creature: Search your library for a land card, put it onto the battlefield tapped, then shuffle.` |
| 3 | Prossh, Skyraider of Kher | `crates/engine/src/cards/defs/prossh_skyraider_of_kher.rs:21` | `Sacrifice another creature: Prossh gets +1/+0 until end of turn.` |
| 4 | Falkenrath Pit Fighter | `crates/engine/src/cards/defs/falkenrath_pit_fighter.rs:16` | `{1}{R}, Discard a card, Sacrifice a Vampire: Draw two cards. Activate only if an opponent lost life this turn.` |
| 5 | Birthing Pod | `crates/engine/src/cards/defs/birthing_pod.rs:20` | `{1}{G/P}, {T}, Sacrifice a creature: Search your library for a creature card with mana value equal to 1 plus the sacrificed creature's mana value …` |
| 6 | Diamond Valley | `crates/engine/src/cards/defs/diamond_valley.rs:19` | `{T}, Sacrifice a creature: You gain life equal to the sacrificed creature's toughness.` |
| 7 | Miren, the Moaning Well | `crates/engine/src/cards/defs/miren_the_moaning_well.rs:31` | `{3}, {T}, Sacrifice a creature: You gain life equal to the sacrificed creature's toughness.` |
| 8 | Priest of Forgotten Gods | `crates/engine/src/cards/defs/priest_of_forgotten_gods.rs:16` | `{T}, Sacrifice two other creatures: …` |
| 9 | Bolas's Citadel | `crates/engine/src/cards/defs/bolass_citadel.rs:29` | `{T}, Sacrifice ten nonland permanents: Each opponent loses 10 life.` |
| 10 | Kellogg, Dangerous Mind | `crates/engine/src/cards/defs/kellogg_dangerous_mind.rs:36` | `Sacrifice five Treasures: Gain control of target creature for as long as you control Kellogg. …` |
| 11 | Grim Hireling | `crates/engine/src/cards/defs/grim_hireling.rs:34` | `{B}, Sacrifice X Treasures: Target creature gets -X/-X until end of turn. …` |
| 12 | Ainok Strike Leader | `crates/engine/src/cards/defs/ainok_strike_leader.rs:26` | `Sacrifice this creature: Creature tokens you control gain indestructible until end of turn.` |

### 2b. Effect-side forced sacrifices ("Each player sacrifices a [filter]")

| # | Card | file:line | Oracle excerpt (MCP) |
|---|------|-----------|----------------------|
| 13 | Fleshbag Marauder | `crates/engine/src/cards/defs/fleshbag_marauder.rs:20` | `When this creature enters, each player sacrifices a creature of their choice.` |
| 14 | Merciless Executioner | `crates/engine/src/cards/defs/merciless_executioner.rs:22` | `When this creature enters, each player sacrifices a creature of their choice.` |
| 15 | Demon's Disciple | `crates/engine/src/cards/defs/demons_disciple.rs:19` | `When this creature enters, each player sacrifices a creature or planeswalker of their choice.` |
| 16 | Accursed Marauder | `crates/engine/src/cards/defs/accursed_marauder.rs:20` | `When this creature enters, each player sacrifices a nontoken creature of their choice.` |
| 17 | Butcher of Malakir | `crates/engine/src/cards/defs/butcher_of_malakir.rs:31` | `Whenever this creature or another creature you control dies, each opponent sacrifices a creature of their choice.` |
| 18 | Dictate of Erebos | `crates/engine/src/cards/defs/dictate_of_erebos.rs:28` | `Whenever a creature you control dies, each opponent sacrifices a creature of their choice.` |
| 19 | Grave Pact | `crates/engine/src/cards/defs/grave_pact.rs:25` | `Whenever a creature you control dies, each other player sacrifices a creature of their choice.` |
| 20 | Anowon, the Ruin Sage | `crates/engine/src/cards/defs/anowon_the_ruin_sage.rs:26` | `At the beginning of your upkeep, each player sacrifices a non-Vampire creature of their choice.` |
| 21 | Liliana, Dreadhorde General (−4) | `crates/engine/src/cards/defs/liliana_dreadhorde_general.rs:56` | `−4: Each player sacrifices two creatures of their choice.` |
| 22 | Blasphemous Edict | `crates/engine/src/cards/defs/blasphemous_edict.rs:18` | `Each player sacrifices thirteen creatures of their choice.` |
| 23 | Vraska's Fall | `crates/engine/src/cards/defs/vraskas_fall.rs:24` | `Each opponent sacrifices a creature or planeswalker of their choice and gets a poison counter.` |
| 24 | Blessed Alliance (mode 2) | `crates/engine/src/cards/defs/blessed_alliance.rs:63` | `Target opponent sacrifices an attacking creature of their choice.` |
| 25 | Crackling Doom | `crates/engine/src/cards/defs/crackling_doom.rs:15` | `Each opponent sacrifices a creature with the greatest power among creatures that player controls.` |
| 26 | Flare of Malice | `crates/engine/src/cards/defs/flare_of_malice.rs:19` | `Each opponent sacrifices a creature or planeswalker with the greatest mana value among creatures and planeswalkers they control.` |
| 27 | By Invitation Only | `crates/engine/src/cards/defs/by_invitation_only.rs:20` | `Choose a number between 0 and 13. Each player sacrifices that many creatures of their choice.` |
| 28 | Ruthless Winnower | `crates/engine/src/cards/defs/ruthless_winnower.rs:21` | `At the beginning of each player's upkeep, that player sacrifices a non-Elf creature of their choice.` |
| 29 | Roiling Regrowth | `crates/engine/src/cards/defs/roiling_regrowth.rs` (whole file blank) | `Sacrifice a land. Search your library for up to two basic land cards …` |

### 2c. Compound / variable sacrifice (multi-sac, X-cost, alt cost)

| # | Card | file:line | Oracle excerpt (MCP) |
|---|------|-----------|----------------------|
| 30 | Scapeshift | `scapeshift.rs:21` | `Sacrifice any number of lands. Search your library for up to that many land cards …` |
| 31 | Last-Ditch Effort | `last_ditch_effort.rs:4` | `Sacrifice any number of creatures. Last-Ditch Effort deals that much damage to any target.` |
| 32 | Dread Return | `dread_return.rs:29` | `Flashback—Sacrifice three creatures.` |
| 33 | Plumb the Forbidden | `plumb_the_forbidden.rs:15` | `As an additional cost to cast this spell, you may sacrifice one or more creatures. …` |
| 34 | Phyrexian Dreadnought | `phyrexian_dreadnought.rs:22` | `When this creature enters, sacrifice it unless you sacrifice any number of creatures with total power 12 or greater.` |
| 35 | Vampire Gourmand | `vampire_gourmand.rs:16` | `Whenever this creature attacks, you may sacrifice another creature. If you do, …` |

The brief's "~14 live TODOs" appears to refer to subset 2b (the Effect-side
forced sacrifices), which lines up almost exactly: 17 entries minus 3 already-
acknowledged-blocked (Crackling Doom / Flare of Malice / By Invitation Only) =
14 candidates. That is the correct planner-claimed count to calibrate against.

---

## 3. Per-card classification

Format: `STATUS — rationale (oracle citation: MCP)`.
The classification answers "would PB-SFT (FIELD-ADDITION on `Effect::SacrificePermanents`,
optionally + 'another' enforcement on `Cost::Sacrifice`) unblock this card cleanly?"

### Effect-side cards (subset 2b — primary PB-SFT scope)

| Card | Status | Rationale |
|---|---|---|
| Fleshbag Marauder | **CONFIRMED-IN-SCOPE** | Filter = `TargetFilter { has_card_type: Creature }`; ETB trigger already wired; `EachPlayer` already supported. Oracle: "each player sacrifices a creature of their choice." |
| Merciless Executioner | **CONFIRMED-IN-SCOPE** | Identical to Fleshbag Marauder oracle text. |
| Demon's Disciple | **CONFIRMED-IN-SCOPE** | Filter = creature OR planeswalker; needs `has_card_types: vec![Creature, Planeswalker]` (TargetFilter already supports). Oracle: "each player sacrifices a creature or planeswalker of their choice." |
| Vraska's Fall | **CONFIRMED-IN-SCOPE** | Same multi-type filter as Demon's Disciple. Oracle: "Each opponent sacrifices a creature or planeswalker of their choice and gets a poison counter." (poison counter half already supported via `AddCounter`.) |
| Butcher of Malakir | **CONFIRMED-IN-SCOPE** | Filter = creature; "this creature or another creature you control dies" trigger already wired (LtB on each-creature-you-control). Oracle: "each opponent sacrifices a creature of their choice." |
| Dictate of Erebos | **CONFIRMED-IN-SCOPE** | Filter = creature; LtB-on-creature-you-control already wired (via Grave Pact dispatch). |
| Grave Pact | **CONFIRMED-IN-SCOPE** | Filter = creature; trigger already shipping. |
| Liliana, Dreadhorde General (−4) | **CONFIRMED-IN-SCOPE** | Filter = creature, `count: Fixed(2)`; loyalty ability + −X dispatch already supported. Oracle: "Each player sacrifices two creatures of their choice." |
| Blasphemous Edict | **CONFIRMED-IN-SCOPE** | Filter = creature, `count: Fixed(13)`; alt-cost reduction "{B} if 13+ creatures" is a separate gap (out of PB-SFT scope), but the main effect with filter ships. |
| Roiling Regrowth | **CONFIRMED-IN-SCOPE** | Filter = land; "Sacrifice a land" at resolution (per ruling: not an additional cost), then SearchLibrary x2 + Shuffle (both already supported). Oracle: "Sacrifice a land. Search your library for up to two basic land cards, put them onto the battlefield tapped, then shuffle." |
| Anowon, the Ruin Sage | **CONFIRMED-IN-SCOPE (borderline)** | Filter = creature with `exclude_subtypes: [Vampire]`; AtBeginningOfYourUpkeep already works for controller (oracle says "each player" but player path through `EachPlayer` works since trigger fires on controller's upkeep). `exclude_subtypes` already in TargetFilter — must be honored at SacrificePermanents resolution selection. |
| Blessed Alliance (mode 2) | **CONFIRMED-IN-SCOPE (borderline)** | Filter = creature + `is_attacking: true`. `is_attacking` is a `GameObject` runtime property, NOT a `Characteristics` field, so resolution-site code must check it explicitly (see TargetFilter doc note). Modal spell already wired. |
| Accursed Marauder | **CONFIRMED-IN-SCOPE (borderline)** | Filter = creature, nontoken. `is_token: bool` in TargetFilter currently means "must be a token"; default `false` means no restriction. PB-SFT must add either an `is_nontoken: bool` mirror field OR re-purpose semantics. Mechanical extension, low risk. |
| Ruthless Winnower | **BLOCKED-BY-OTHER-PRIMITIVE** | Filter = non-Elf creature works, BUT trigger is "At the beginning of each player's upkeep" — current engine only supports `AtBeginningOfYourUpkeep` (controller's). `AtBeginningOfEachPlayersUpkeep` is a separate trigger-condition primitive gap. |
| Crackling Doom | **OUT-OF-SCOPE** | "Sacrifice a creature with the greatest power among creatures that player controls" is a **selection rule**, not a static filter — picks the max-power permanent per controller. Different primitive (greatest-X-among selection). |
| Flare of Malice | **OUT-OF-SCOPE** | Same as Crackling Doom (greatest-MV-among) plus alt cost (sacrifice nontoken black creature instead of mana). Two separate primitives. |
| By Invitation Only | **OUT-OF-SCOPE** | Variable count chosen by caster ("a number between 0 and 13") — needs a player-choice cost mechanic; the count itself is a free choice, not an EffectAmount expression. |

### Cost-side cards (subset 2a — secondary PB-SFT scope, "another" enforcement)

| Card | Status | Rationale |
|---|---|---|
| Commissar Severina Raine | **CONFIRMED-IN-SCOPE (Cost path)** | Activated ability `{2}, Sacrifice another creature: gain 2 life, draw card` is fully expressible as `Cost::Sacrifice(TargetFilter::Creature)` + GainLife + DrawCards (all supported) — PROVIDED `Cost::Sacrifice` enforces "another" (does not currently). Attack trigger half is blocked by EffectAmount::AttackingCreatureCount but the activated half is independently shippable; activated and triggered abilities are added separately. |
| Wight of the Reliquary | **BLOCKED-BY-OTHER-PRIMITIVE** | Activated ability is fine + would benefit from "another" enforcement, BUT the CDA "+1/+1 for each creature card in your graveyard" is unimplementable without `EffectAmount::CountInGraveyard` / dynamic P/T modification. W5 policy disallows partial card defs that produce wrong P/T. |
| Prossh, Skyraider of Kher | **BLOCKED-BY-OTHER-PRIMITIVE** | Sac filter = creature/another fine, BUT effect "+1/+0 EOT" needs `Effect::PumpUntilEndOfTurn` (or layer-3 timestamped P/T modifier) which is not in DSL. Cast trigger half also blocked (X-equals-mana-spent). |
| Falkenrath Pit Fighter | **BLOCKED-BY-OTHER-PRIMITIVE** | Sac filter = `SacrificeFilter::Subtype("Vampire")` already supported, BUT activation condition "only if an opponent lost life this turn" needs `Condition::OpponentLostLifeThisTurn` (not in DSL). |
| Birthing Pod | **BLOCKED-BY-OTHER-PRIMITIVE** | Sac filter = creature fine, BUT "search for creature with mana value equal to 1 plus the sacrificed creature's mana value" needs dynamic-MV-from-LKI search filter (not in DSL). PhyrexianMana cost already supports `{G/P}` (PB-9). |
| Diamond Valley | **BLOCKED-BY-OTHER-PRIMITIVE** | Sac filter = creature fine, BUT "gain life equal to the sacrificed creature's toughness" needs `EffectAmount::SacrificedCreatureToughness` (LKI). PB-P shipped sacrificed-creature-power; toughness-LKI is a parallel addition not yet done. |
| Miren, the Moaning Well | **BLOCKED-BY-OTHER-PRIMITIVE** | Same toughness-LKI gap as Diamond Valley. |
| Priest of Forgotten Gods | **BLOCKED-BY-OTHER-PRIMITIVE** | "Sacrifice **two** other creatures" — multi-permanent activation cost is NOT in `Cost::Sacrifice` (single sac_id only). Distinct primitive. Plus complex multi-target effect on top. |
| Bolas's Citadel | **BLOCKED-BY-OTHER-PRIMITIVE** | "Sacrifice ten nonland permanents" — multi-sac + nonland filter (TargetFilter has `non_land`, but lowering to `SacrificeFilter` enum drops it). Multi-sac primitive missing. |
| Kellogg, Dangerous Mind | **BLOCKED-BY-OTHER-PRIMITIVE** | "Sacrifice five Treasures" — multi-sac + Subtype already in enum, BUT effect "Gain control of target creature for as long as you control Kellogg" needs conditional duration (PB-control gap). |
| Grim Hireling | **BLOCKED-BY-OTHER-PRIMITIVE** | "Sacrifice X Treasures" — X-cost variable sac + variable -X/-X effect amount. Two gaps. |
| Ainok Strike Leader | **OUT-OF-SCOPE** | Sac is `SacrificeSelf` (already supported); blocker is the grant effect "creature tokens you control gain indestructible until end of turn" (PB-6 grant filter primitive). |

### Compound / variable / alt-cost cards (subset 2c)

All are **OUT-OF-SCOPE for PB-SFT** — they need at least one of:
multi-sac variable count (`Cost::SacrificeAnyNumber`),
sacrificed-count tracking (`EffectAmount::SacrificedCount`),
optional/may-sacrifice in reflexive trigger,
or alt-cost-as-sacrifice (`AltCostKind::SacrificeCreature`).

Listed in the stop-and-flag log (§5) for visibility.

---

## 4. Yield calibration

Reference: `~/.claude/projects/-home-skydude-projects-scutemob/memory/feedback_pb_yield_calibration.md`.

PB-SFT slots into the **filter-PB** category — it adds a filter field to an
existing effect / cost variant. The category midpoint is **50–65%** (PB-X 75 %,
PB-S 71 %, PB-T 64 %).

| Metric | Value |
|--------|-------|
| Planner-claimed in-scope (per task brief) | **~14** (subset 2b TODOs) |
| Re-triage CONFIRMED-IN-SCOPE (Effect side, clean) | 10 (Fleshbag, Merciless Executioner, Demon's Disciple, Vraska's Fall, Butcher of Malakir, Dictate of Erebos, Grave Pact, Liliana DH-4, Blasphemous Edict, Roiling Regrowth) |
| Re-triage CONFIRMED-IN-SCOPE (Effect side, borderline) | 3 (Anowon, Blessed Alliance mode 2, Accursed Marauder) |
| Re-triage CONFIRMED-IN-SCOPE (Cost side, optional roll-in) | 1 (Commissar Severina Raine) |
| Re-triage BLOCKED / OUT-OF-SCOPE | 4 cost-side multi-sac / X-cost / alt-cost (Bolas's Citadel, Priest of Forgotten Gods, Kellogg, Grim Hireling) + 3 effect-side selection-rule (Crackling Doom, Flare of Malice, By Invitation Only) + 6 cost-side blocked on a sibling primitive (Wight, Prossh, Falkenrath, Birthing Pod, Diamond Valley, Miren) + 6 compound/variable (subset 2c) |
| 50–65% midpoint × 14 | **7–9 expected to ship after review** |
| 50–65% midpoint × 13 (10 clean + 3 borderline) | **7–9** |

**Calibrated yield estimate: 7–9 cards ship from PB-SFT.** 4 of the 13 are
"clean+borderline-but-need-extra-engine-touch" (Anowon: exclude_subtypes honored;
Blessed Alliance: is_attacking honored at sac selection; Accursed Marauder:
is_nontoken added; Liliana DH-4: ensure count > 1 selects deterministically and
respects filter on each pick). Conservative read: **5–6 clean + 2–3 borderline = 7–9.**

Trigger-PB rate of 15–25 % (PB-N) does **not** apply here — this is a filter
field addition, not a new TriggerCondition.

---

## 5. Stop-and-flag log: cards touching other unimplemented primitives

These cards have a sacrifice-filter TODO but are **blocked by at least one
sibling primitive** that PB-SFT alone will not unblock. They should NOT be re-
authored as part of the PB-SFT batch even though their sacrifice line goes from
broken to authorable; their other gaps would still produce wrong game state.

| Card | Sibling primitive blocker(s) |
|------|-------------------------------|
| Wight of the Reliquary | `EffectAmount::CountInGraveyard` for CDA "+1/+1 per creature card in graveyard" |
| Prossh, Skyraider of Kher | `Effect::PumpUntilEndOfTurn`; cast-trigger with `EffectAmount::ManaSpentToCast` |
| Falkenrath Pit Fighter | `Condition::OpponentLostLifeThisTurn` activation gate |
| Birthing Pod | `TargetFilter::min_cmc / max_cmc` relative to LKI sacrificed permanent (dynamic MV reference) |
| Diamond Valley | `EffectAmount::SacrificedCreatureToughness` LKI |
| Miren, the Moaning Well | same as Diamond Valley |
| Priest of Forgotten Gods | multi-sac (count > 1) `Cost::Sacrifice` variant; multi-target life-loss-then-sacrifice |
| Bolas's Citadel | multi-sac (count = 10); `non_land` honored in sacrifice filter |
| Kellogg, Dangerous Mind | multi-sac (count = 5); `Effect::GainControlForAsLongAs(condition)` |
| Grim Hireling | X-cost variable sacrifice; `EffectAmount::XValue` on -X/-X target effect |
| Ainok Strike Leader | sacrifice is self (works); blocked on PB-6 grant filter "creature tokens gain indestructible until EOT" |
| Vampire Gourmand | reflexive optional sacrifice ("you may sacrifice…if you do, …") + `Effect::CantBeBlockedThisTurn` |
| Korvold, Fae-Cursed King | forced-sacrifice-of-another-permanent on ETB+attack triggers; "whenever you sacrifice a permanent" trigger |
| Smothering Abomination | "At upkeep, sacrifice a creature" forced (no cost path); "whenever you sacrifice creature" trigger |
| Sheoldred, Whispering One | `AtBeginningOfEachOpponentsUpkeep` trigger gap |
| Ruthless Winnower | `AtBeginningOfEachPlayersUpkeep` trigger gap |
| Ziatora, the Incinerator | reflexive optional sacrifice + sacrificed-power-LKI (PB-P delivered for cost path; reflexive-trigger path may need re-wire) |
| Ruthless Lawbringer | reflexive optional sacrifice + targeted destroy permanent |
| Springbloom Druid | optional ETB sacrifice ("you may sacrifice a land. If you do, search…") |
| Phyrexian Dreadnought | conditional ETB sacrifice ("sacrifice unless you sacrifice creatures totaling power 12+") |
| Crackling Doom | greatest-power-among selection rule |
| Flare of Malice | greatest-MV-among selection rule + AltCostKind::SacrificeCreature |
| By Invitation Only | player-choice variable count (0–13) |
| Scapeshift, Last-Ditch Effort | `Cost::SacrificeAnyNumber` variable count + `EffectAmount::SacrificedCount` |
| Dread Return | Flashback alt cost = multi-sac (sacrifice three creatures) |
| Plumb the Forbidden | additional cost = variable sacrifice + `EffectAmount::SacrificedCount` for spell copies |

Reviewer agent should NOT request these cards be re-authored as part of PB-SFT.

---

## 6. Dispatch-ready scope

### 6a. Engine surface changes

1. **`crates/engine/src/cards/card_definition.rs:1503-1506`** — extend
   `Effect::SacrificePermanents`:
   ```rust
   SacrificePermanents {
       player: PlayerTarget,
       count: EffectAmount,
       /// PB-SFT (CR 701.17a + 109.1c): optional filter on which permanents the
       /// player must sacrifice. None = any permanent (existing behavior).
       /// All declared filter fields must match the layer-resolved characteristics
       /// at resolution time.
       #[serde(default)]
       filter: Option<TargetFilter>,
   }
   ```
2. **Resolution site** for `Effect::SacrificePermanents` in
   `crates/engine/src/effects/sacrifice.rs` (or wherever the dispatch lives —
   `rust-analyzer references` on the variant will pinpoint it; expected ~1
   resolver function). Apply `filter` to the candidate-permanent list before
   the deterministic min-ObjectId selection. Use `calculate_characteristics()`
   for filter matching (CR 613.1f), with fallbacks for `is_token` /
   `is_attacking` / `is_nontoken` (the runtime-property fields documented in
   TargetFilter to be checked at the call site).
3. **Optional roll-in (Cost-side "another" enforcement)** — `crates/engine/src/rules/abilities.rs:675-741`:
   add a check that `sac_id != source` after the existing zone/controller
   validation. Rejects with `InvalidCommand("sacrifice cost: cannot sacrifice the
   activating permanent (CR 602.2: 'another' filter)")`. ~3 lines. Without this,
   Commissar Severina Raine ships with the same correctness gap Wight of the
   Reliquary already has — defensible to do separately, but it's tiny.

No new variants on `Cost`, `SacrificeFilter`, or `AbilityDefinition`.

### 6b. Dispatch sites

| Surface | Sites |
|---------|-------|
| `Effect::SacrificePermanents { … }` literal construction | **14 cards** (anowon, blasphemous_edict, blessed_alliance, butcher_of_malakir, demons_disciple, dictate_of_erebos, fleshbag_marauder, flare_of_malice, grave_pact, liliana_dreadhorde_general, merciless_executioner, ruthless_winnower, vraskas_fall, accursed_marauder) — each needs `filter: None` (or the new filter) added |
| Resolution / hash / serialization sites | 1 resolution site, 1 `HashInto` arm, plus auto-derived Serde |
| Tests | new file `crates/engine/tests/effect_sacrifice_permanents_filter.rs` |

Estimated total dispatch-touch sites: **~16 files**. Construction-site updates
are pure mechanical adds (`..Default::default()` continues to work for any def
that already uses struct shorthand).

### 6c. Mandatory tests count

**5 mandatory tests** in a new integration test file:

1. `test_each_player_sacrifices_creature_filter` — Fleshbag Marauder pattern;
   each player has a creature + a non-creature permanent; verify the creature
   is the one sacrificed.
2. `test_each_player_sacrifices_land_filter` — Roiling Regrowth pattern; player
   has lands and creatures; verify the land is sacrificed.
3. `test_multi_count_sacrifice_with_filter` — Liliana DH-4 pattern (count = 2);
   player has 3 creatures + 1 artifact; verify the 2 lowest-ObjectId creatures
   are sacrificed and the artifact is untouched.
4. `test_filter_excludes_all_player_has_nothing_to_sacrifice` — player matches
   filter for zero permanents; verify no sacrifice occurs and no error
   (CR 701.17a: "If the player controls fewer than `count`, they sacrifice all").
5. `test_multi_type_filter_creature_or_planeswalker` — Vraska's Fall / Demon's
   Disciple pattern; player has creature + planeswalker + artifact; verify
   either creature or planeswalker is sacrificed (deterministic min-ObjectId)
   but never the artifact.

Optional 6th test if "another" enforcement rolls in:
6. `test_cost_sacrifice_does_not_accept_source` — try to activate
   Commissar Severina Raine while sacrificing herself; expect `InvalidCommand`.

### 6d. Risk

- **Low.** Field addition with `#[serde(default)] Option<TargetFilter>`. Existing
  scripts with `Effect::SacrificePermanents { player, count }` deserialize as
  `filter: None` (no behavior change).
- One pre-existing-bug class: `is_attacking` / `is_token` are runtime fields, not
  characteristics. The filter check at the SacrificePermanents resolution site
  must explicitly handle them OR document that these flags are ignored for now.
  Recommend: implement `is_attacking` (cheap; needed by Blessed Alliance) and
  defer `is_nontoken` (Accursed Marauder remains BLOCKED until added).
- The "another" enforcement is a behavior change for Wight of the Reliquary
  (currently allows sacrificing self per its own TODO). Acceptable because the
  current behavior is wrong; the W5 audit already flagged it.

### 6e. Estimated effort

**1 implementation session** (Sonnet runner): add field, wire resolution,
update 14 construction sites with `filter: None` (or fill in filter for the 7
clean cards in this batch), write 5 tests, run `cargo test --all` and
`cargo clippy -- -D warnings`. Total ~3–4 hours wall.

**1 review session** (Opus reviewer): verify CR 701.17a semantics and the layer-
resolved filter check. ~1 hour.

**Yield**: 7–9 cards re-authored as part of the batch (Fleshbag Marauder,
Merciless Executioner, Butcher of Malakir, Dictate of Erebos, Grave Pact,
Liliana DH-4, Blasphemous Edict, Roiling Regrowth, Vraska's Fall ± Demon's
Disciple ± Anowon ± Blessed Alliance ± Accursed Marauder).

---

## 7. Worker self-confidence notes

- **MCP authoritative**: every oracle citation in §2 came from
  `mcp__mtg-rules__lookup_card`; comments in card defs were cross-checked but
  not relied on as the source of truth.
- **Engine surface verified**: `grep -rn "sacrifice_filter\|Cost::Sacrifice"`
  + `grep -rn "Effect::SacrificePermanents"` walked end-to-end through
  `card_definition.rs` → `replay_harness.rs:flatten_cost_into` →
  `abilities.rs:enforce_sacrifice_cost` → `hash.rs:HashInto` to confirm the
  full dispatch chain, per `feedback_verify_full_chain.md`.
- **Calibration applied conservatively**: 14 → 7–9 expected, in the middle of
  the filter-PB band.
- **Reframing flagged explicitly**: the planner's title `Cost::SacrificeFilteredType`
  is misleading; verdict reframes to FIELD-ADDITION on `Effect::SacrificePermanents`.
- The 3 callsites at `card_definition.rs:3082/3131/3183` are correct as-written
  (token specs with sacrifice_self); they are not the wire-up sites the brief
  implies.

---

## 8. Verdict (repeated at end for grep-ability)

**PROCEED — FIELD-ADDITION on `Effect::SacrificePermanents` (add `filter: Option<TargetFilter>`).**
Optional roll-in: 3-line "another" enforcement on `Cost::Sacrifice` in
`abilities.rs:675-741`. Calibrated yield 7–9 cards. 5 mandatory tests.
~16 dispatch sites. Low risk.
