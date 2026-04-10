# A-42 Tier 4 Diagnosis

> **Date**: 2026-04-10
> **Purpose**: Scrutinize each "blocked" Tier 4 card before building PBs in the wrong order.
> **Method**: Read oracle text, grep existing primitives, identify exact blocker.

## Angrath's Marauders verification

✅ **CORRECT**. Already authored at `crates/engine/src/cards/defs/angraths_marauders.rs`.
Uses `DamageTargetFilter::FromControllerSources(PlayerId(0))` — this filter matches on the
source side only (any target is fine), which exactly matches the oracle "If a source you
control would deal damage to a permanent or player, it deals double that damage instead."
The PlayerId(0) placeholder is bound at registration. No fix needed.

---

## Per-card diagnosis

### 1. Patriarch's Bidding — `{4}{B}{B}`, Sorcery

**Oracle**: Each player chooses a creature type. Each player returns all creature cards of a type chosen this way from their graveyard to the battlefield.

- **Blocker**: Per-player interactive type choice. Then per-player mass reanimate filtered by the union of all chosen types.
- **Existing primitives?** PB-H `ReturnAllFromGraveyardToBattlefield` handles mass GY→BF for one graveyard scope. Choose-type exists (PB-D) but only as a *spell-level* single choice (Kindred Dominance), not per-player.
- **Verdict**: **Tier 4c (M10-blocked)** on interactivity. Deterministic fallback is ugly (each player picks the type with most cards in their own GY). Could be done deterministically as ~60 LOC: `Effect::PatriarchsBidding` special-case. Not a reusable primitive — carve as one-off.
- **CR**: 101.4, 603.1, 613 (per-player choice ordering).

### 2. Deepglow Skate — `{4}{U}`, Creature 3/3

**Oracle**: When this creature enters, double the number of each kind of counter on any number of target permanents.

- **Blocker**: `ReplacementModification::DoubleCounters` exists but it's a **replacement on "would place counters" events** (Vorinclex shape). Deepglow Skate is an **instant effect on existing counters** — different primitive.
- **Missing**: `Effect::DoubleCountersOnTarget { target: EffectTarget }` that reads current counter counts and adds matching counters.
- **Est. LOC**: ~40 (Effect variant + execute + hash + test + up-to-N targets is separate issue).
- **Additional gap**: "any number of target permanents" (up-to-N targets) — not in targeting. Deepglow can approximate as single-target for now.
- **Verdict**: **Tier 4b — single micro-PB** ("PB-R: DoubleCountersOnTarget"). Single-target version is ~40 LOC. Up-to-N is a separate generic gap that unblocks many cards.
- **CR**: 121.6.

### 3. Breach the Multiverse — `{5}{B}{B}`, Sorcery

**Oracle**: Each player mills ten cards. For each player, choose a creature or planeswalker card in that player's graveyard. Put those cards onto the battlefield under your control. Then each creature you control becomes a Phyrexian in addition to its other types.

- **Blocker**: (a) per-player reanimate-one-choice, (b) type-addition to creatures you control.
- **Existing primitives?** `MillCards` exists with `ForEach::EachPlayer`. `ReturnAllFromGraveyardToBattlefield` could pick one per GY if filter supported "first creature/PW in each player's GY" — not quite what we need (interactive choice per player).
- **Missing**: `Effect::ForEachPlayerChooseAndReanimate { filter, controller_override }` OR interactive choice (M10). Also type-addition (AddCreatureType — may exist as a layer effect).
- **Verdict**: **Tier 4c (M10 interactive)** on the choice part. Could deterministically pick highest mana-value per GY as a fallback. ~80 LOC for deterministic version. Carve as one-off effect.
- **CR**: 101.4, 613.1d.

### 4. New Blood — `{2}{B}{B}`, Sorcery

**Oracle**: As an additional cost to cast this spell, tap an untapped Vampire you control. Gain control of target creature. Change the text of that creature by replacing all instances of one creature type with Vampire.

- **Blocker**: **Text-changing effect** (CR 613.1e, sub-layer 3). NO text-change primitive exists anywhere in the engine. This is a substantial new subsystem.
- **Existing primitives?** `Effect::GainControl` exists. Tap-creature-of-type cost exists (PB-4 sac, tap-as-cost may exist).
- **Missing**: `LayerModification::ReplaceCreatureType { from: SubType, to: SubType }` applied in Layer 3 (text-changing effects).
- **Est. LOC**: ~100 (new Layer 3 modification + choose-creature-type on cast + test).
- **Verdict**: **Tier 4b — dedicated PB ("PB-S: Text-changing effects")**. Would also unblock Mind Control variants, Artificial Evolution, Conspiracy, etc. Probably worth doing but scope is medium.
- **CR**: 612, 613.1e.

### 5. Treasure Nabber — `{3}{R}`, Creature 3/3

**Oracle**: Whenever an opponent taps an artifact for mana, gain control of that artifact until the end of your next turn.

- **Blocker**: (a) `WhenOpponentTapsArtifactForMana` trigger variant, (b) gain-control-until-end-of-your-next-turn duration.
- **Existing primitives?** PB-E added `WhenTappedForMana` (for Mana Reflection etc.) — triggered on SELF being tapped for mana. Treasure Nabber needs the inverse: trigger on **any** opponent's artifact being tapped for mana.
- ✅ `EffectDuration::UntilYourNextTurn(PlayerId)` EXISTS (grep confirmed). Duration is available.
- ✅ `Effect::GainControl { target, duration }` EXISTS.
- **Missing**: One new trigger variant `TriggerCondition::WheneverOpponentTapsArtifactForMana` (or a more general `OpponentTapsPermanentForMana` with `ObjectFilter`).
- **Est. LOC**: ~25 (trigger variant + fire site in tap_for_mana path + hash + test). **SMALL.**
- **Verdict**: **Tier 4b — micro-PB** (add to same PB as Roaming Throne as "PB-trigger-extensions"). Could also be authored standalone.
- **CR**: 603.2.

### 6. Ghyrson Starn, Kelermorph — `{1}{U}{R}`, Creature 2/2

**Oracle**: Ward {2}. Three Autostubs — Whenever another source you control deals exactly 1 damage to a permanent or player, Ghyrson Starn deals 2 damage to that permanent or player.

- **Blocker**: This is a **triggered ability on a damage-dealt event with an amount-equals-1 condition**, NOT a replacement effect. I mis-classified in Tier 4.
- **Existing primitives?** Ward exists. `TriggerCondition::WhenDealsDamage` or similar may exist. Need "other source you control" + "amount == 1" filter + "deal that much damage to same target" sub-effect.
- **Missing**: `TriggerEvent::SourceYouControlDealsExactlyN { n: u32, excluding_self: bool }` + an effect that targets the "damaged thing" from context (may need damage-event context threading).
- **Est. LOC**: ~50 (new trigger variant + context capture + test).
- **Verdict**: **Tier 4b — micro-PB**. Fairly contained. Could go into a "PB-damage-triggers" batch.
- **CR**: 603.2, 702.21 (Ward).

### 7. Marvin, Murderous Mimic — `{1}{U}`, Creature 1/2

**Oracle**: Marvin has all activated abilities of creatures you control that don't have the same name as this creature.

- **Blocker**: Reflection — dynamically copies activated abilities from other permanents by name-differential. Requires `LayerModification::GrantActivatedAbility` (confirmed gap by Citanul Hierophants, Cryptolith Rite, Chromatic Lantern, Paradise Mantle, Umbral Mantle, Enduring Vitality TODOs).
- **Existing primitives?** `GrantKeyword` exists in LayerModification. `GrantActivatedAbility` is a **known pervasive gap**.
- **Missing**: `LayerModification::GrantActivatedAbility { filter: ObjectFilter, abilities: Vec<AbilityDefinition> }` + a dynamic variant for "all activated abilities of other creatures filtered by name-differential".
- **Est. LOC**: ~150 for basic static grant (affects 7+ cards). Marvin's name-differential reflection is an additional ~50 LOC.
- **Verdict**: **Tier 4b — dedicated PB ("PB-T: GrantActivatedAbility")**. **HIGH PRIORITY** — pervasive gap, unblocks 7+ cards (Cryptolith Rite, Citanul Hierophants, Chromatic Lantern, Paradise Mantle, Umbral Mantle, Enduring Vitality, Marvin, Song of Freyalise).
- **CR**: 613.1f, 702.

### 8. Time Spiral — `{4}{U}{U}`, Sorcery

**Oracle**: Exile Time Spiral. Each player shuffles their hand and graveyard into their library, then draws seven cards. You untap up to six lands.

- **Blocker**: (a) `Effect::ShuffleHandAndGraveyardIntoLibrary { player: ForEach::EachPlayer }`, (b) up-to-N target untap, (c) `self_exile_on_resolution` already exists (✅ PB-C).
- **Existing primitives?** Winds of Change TODO confirms `ShuffleHandIntoLibrary` gap. Up-to-N targets is a known general gap.
- **Missing**: `Effect::ShuffleZonesIntoLibrary { zones: Vec<ZoneType>, player: PlayerTarget }` (~30 LOC). Up-to-N untap target is the harder bit (~60 LOC of targeting + harness support).
- **Verdict**: **Tier 4b — ShuffleZonesIntoLibrary is a small micro-PB** that also unblocks Winds of Change, Timetwister, Memory's Journey, etc. Up-to-N is a separate general gap. Time Spiral could be approximated as single-target untap (lose the "up to 6") until the general gap is fixed.
- **CR**: 701.19.

### 9. Morality Shift — `{4}{B}{B}`, Sorcery

**Oracle**: Exchange your graveyard and library. Then shuffle your library.

- **Blocker**: Zone exchange. No `Effect::ExchangeZones` primitive.
- **Est. LOC**: ~25 (literally swap GY and library Vec contents, then shuffle).
- **Verdict**: **Tier 4b — trivial micro-PB**. ~25 LOC. Possibly the cheapest unblock in the whole list.
- **CR**: 701.8, 701.19.

### 10. Crackle with Power — `{X}{R}{R}{R}`, Sorcery

**Oracle**: Crackle with Power deals five times X damage to each of up to X targets.

- **Blocker**: (a) up-to-X targets (variable target count tied to cost), (b) per-target scaling damage (5×X).
- **Existing primitives?** PB-27 has X-cost spells. Damage scaling via `EffectAmount::XValue` exists. Up-to-N targets is the pervasive gap.
- **Missing**: **Up-to-N targets** where N is tied to the X value. This is the **same underlying gap** as Time Spiral and Deepglow Skate.
- **Verdict**: **Tier 4b — blocked on shared gap (up-to-N targets)**. Would need `TargetRequirement::UpTo { max: EffectAmount }`.
- **CR**: 601.2c, 107.3.

---

## Re-bucketed Classification

### Tier 4a — Surprise wins (already authorable)
**None.** All 10 cards genuinely need engine work.

### Tier 4b — Single-primitive-batch unblocks (8 cards, 4 suggested PBs)

| PB | Unblocks | Scope | Priority |
|----|----------|-------|----------|
| **PB-R: ExchangeZones + ShuffleZones** | Morality Shift, Time Spiral (partial), Winds of Change, Timetwister | ~60 LOC | **HIGH** (trivial, unblocks many) |
| **PB-S: GrantActivatedAbility** | Marvin, Cryptolith Rite, Citanul Hierophants, Chromatic Lantern, Paradise Mantle, Umbral Mantle, Enduring Vitality, Song of Freyalise (8+ cards) | ~150-200 LOC | **VERY HIGH** (pervasive gap, most card yield) |
| **PB-T: Up-to-N targeting** | Deepglow Skate, Crackle with Power, Time Spiral (fully), Scour from Existence, many others | ~100 LOC targeting + harness | **HIGH** (pervasive gap) |
| **PB-U: Trigger extensions** | Treasure Nabber, Ghyrson Starn, Roaming Throne (from Tier 2) | ~75 LOC total | **MEDIUM** (three cards per session) |
| **PB-V: DoubleCountersOnTarget** | Deepglow Skate (combine with PB-T) | ~40 LOC | **LOW** (one card) |
| **PB-W: Text-changing effects** | New Blood, Artificial Evolution, Mind Bend, Conspiracy (~5 cards) | ~100 LOC | **LOW** (small yield) |

### Tier 4c — Genuinely hard or M10-blocked (2 cards)

| Card | Why |
|------|-----|
| **Patriarch's Bidding** | Per-player interactive choice with cross-player correlation. Deterministic fallback is acceptable but one-off (not reusable). |
| **Breach the Multiverse** | Per-player interactive "choose one to reanimate". Deterministic fallback acceptable but one-off. |

Both could be done as special-case one-off Effect variants (~60-80 LOC each), deferring real choice to M10. Call these **Tier 4c.1** — solvable without M10 via deterministic fallback, acceptable for single-player/bot play, surface in review for M10 upgrade.

---

## Revised recommended order

1. **Complete Tier 1 authoring (8 cards)** — still the cheapest next step, zero engine work.
2. **PB-R: Zone swaps** (~60 LOC) — cheapest engine unblock, trivial risk, +2 cards (Morality Shift, Time Spiral-partial) plus ~3 cards elsewhere (Winds of Change, Timetwister).
3. **PB-S: GrantActivatedAbility** (~150-200 LOC) — **HIGHEST YIELD** engine work across the entire codebase. Unblocks 8+ cards including Marvin plus 7 others that have been TODO for months.
4. **PB-T: Up-to-N targeting** (~100 LOC) — generic unblock, feeds Crackle with Power, Deepglow Skate (with PB-V), Time Spiral, and others.
5. **PB-U: Trigger extensions** (Treasure Nabber + Ghyrson Starn + Roaming Throne) — one small session, 3 cards.
6. **PB-V: DoubleCountersOnTarget** (~40 LOC) — combine with PB-T session to finish Deepglow Skate.
7. **Tier 2 cards** (Banner of Kinship, Mana Geyser, Auriok Steelshaper, Faith's Reward) — author alongside their unblocking PBs.
8. **Tier 3 (PB-Q: ChooseColor)** — unblocks Caged Sun, Gauntlet of Power, Utopia Sprawl, High Tide (partial), Throne of Eldraine, Nykthos, Skrelv, Contamination, Prismatic Omen — **EVEN HIGHER YIELD than I thought** once cross-referenced.
9. **PB-W: Text-changing effects** — lowest yield, defer.
10. **Tier 4c** (Patriarch's Bidding, Breach the Multiverse) — one-off special-case Effects after above, or defer to M10.

## Key insights

- **PB-S (GrantActivatedAbility) is the biggest lever.** Not because of Marvin — because of the 7 other cards blocked on the same gap (Cryptolith Rite, Citanul Hierophants, Chromatic Lantern, Paradise Mantle, Umbral Mantle, Enduring Vitality, Song of Freyalise). These are popular cards that have been silently blocked.
- **Up-to-N targeting is a pervasive general gap.** Deepglow Skate, Crackle with Power, Time Spiral all share it. Worth doing as a focused session.
- **No Tier 4 card was mis-classified into Tier 1/2.** The initial Tier 4 bucket held up. But a few are **cheaper than I originally estimated** (Morality Shift, Treasure Nabber, Ghyrson Starn).
- **Tier 4c is only 2 cards** (was 10 before diagnosis). The Tier 4 bucket collapses down to 2 real M10-style blockers.
