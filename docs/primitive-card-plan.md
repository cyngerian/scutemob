# Primitive-First Card Plan

> **Replaces** the wave-based authoring approach in `test-data/test-cards/AUTHORING_PLAN.md`.
> Each batch implements a DSL primitive in the engine, then immediately fixes/authors all
> cards it unblocks. No card is authored until its required primitives exist.
>
> **Goal**: Every card complete pre-alpha. No TODOs, no partial implementations, no wrong game state.
>
> **Source audit**: `memory/card-authoring/dsl-gap-audit.md`

---

## Current State

- **718** card defs exist
- **418** (58%) have TODO comments
- **122** produce actively wrong game state
- **~1,000** cards remaining in the 1,743-card authoring universe
- **7** cards deferred post-alpha (Planeswalker, Saga/Class, Meld)

---

## Batch Overview

| Batch | Primitive | Cards Fixed/Unblocked | Sessions | Dependencies |
|-------|-----------|----------------------:|----------|-------------|
| PB-0 | Quick wins (no engine changes) | 23 | 1 | None |
| PB-1 | Mana with damage (pain lands) | 8 | 1 | None |
| PB-2 | Conditional ETB tapped | 56 | 2-3 | None |
| PB-3 | Shockland ETB (pay-or-tapped) | 10 | 1 | None |
| PB-4 | Sacrifice as activation cost | 26 | 2 | None |
| PB-5 | Targeted activated/triggered abilities | 32 | 2-3 | None |
| PB-6 | Static grant with controller filter | 30 | 1-2 | None |
| PB-7 | Count-based scaling | 29 | 2 | PB-6 partial |
| PB-8 | Cost reduction statics | 10 | 1-2 | PB-6 |
| PB-9 | Hybrid mana & X costs | 7 | 1-2 | None |
| PB-9.5 | Architecture cleanup (no new DSL) | 0 | 1 | PB-9 |
| PB-10 | Return from zone effects | 8 | 1 | PB-5 |
| PB-11 | Mana spending restrictions + ETB choice | 13 | 2 | None |
| PB-12 | Complex replacement effects | 11 | 2-3 | None |
| PB-13 | Specialized mechanics (10 sub-batches) | 19 | 3-4 | None |
| PB-14 | Planeswalker support + emblems | 31+ | 4-6 | None |
| PB-15 | Saga & Class mechanics | 3+ | 2-3 | None |
| PB-16 | Meld | 1 | 1-2 | None |
| PB-17 | Library search filters (non-basic) | 74 | 3-4 | PB-5 |
| PB-18 | Stax / action restrictions | 13 | 2 | PB-6 |
| PB-19 | Mass destroy / board wipes | 12 | 1-2 | None |
| PB-20 | Additional combat phases | 10 | 2 | None |
| PB-21 | Fight & Bite | 5+ | 1 | PB-5 |
| **Phase 1 Total** | | **~400+ existing** | **42-60** | |

After all primitives: every card in the 1,743-card universe is fully expressible.
Phase 2 authors the remaining ~1,025 cards. Phase 3 audits for zero TODOs.

**Key finding from unauthored card analysis** (1,195 cards scanned):
- Planeswalkers: **31 cards** (was estimated at 4 — PB-14 is much larger)
- Library search filters: **74 cards** — the #1 new gap, not in original plan
- Proliferate: 26, exile-and-play: 17, damage-each-opponent: 16 — all need wiring verification
- Stax restrictions: 13 — entirely new framework needed

---

## Recommended Execution Order

Ordered by safety-criticality, then impact:

1. **PB-0** — Quick wins (23 cards, 1 session)
2. **PB-1** — Pain lands (8 cards, 1 session) — safety-critical
3. **PB-5** — Targeted abilities (32 cards, 2-3 sessions) — highest leverage
4. **PB-2** — Conditional ETB (56 cards, 2-3 sessions) — most cards
5. **PB-6** — Static grants (30 cards, 1-2 sessions)
6. **PB-4** — Sacrifice cost (26 cards, 2 sessions)
7. **PB-3** — Shockland ETB (10 cards, 1 session)
8. **PB-7** — Count scaling (29 cards, 2 sessions)
9. **PB-8** — Cost reduction (10 cards, 1-2 sessions)
10. **PB-9** — Hybrid/X (7 cards, 1-2 sessions)
11. **PB-9.5** — Architecture cleanup (0 cards, 1 session) — trigger flush + test file defaults
12. **PB-10** — Return from zone (8 cards, 1 session)
12. **PB-11** — Mana restriction + ETB choice (13 cards, 2 sessions)
13. **PB-12** — Complex replacements (11 cards, 2-3 sessions)
14. **PB-13** — Specialized mechanics (19 cards, 3-4 sessions)

---

## Batch Details

### PB-0: Quick Wins (23 cards, 0 engine changes, 1 session)

Zero new DSL work needed. Fix immediately.

| Fix | Cards | Action |
|-----|-------|--------|
| Simple `etb_tapped` missing | 12 | Add ETB tapped replacement (pattern exists) |
| Cycling not wired | 5 | Add `Cycling { cost }` + `Keyword(Cycling)` — DSL already supports it |
| Missing Flying | 1 | thousand_faced_shadow: add `Keyword(Flying)` |
| Color indicator | 1 | dryad_arbor: set `color_indicator` |
| Wither keyword | 1 | boggart_ram_gang: add `KeywordAbility::Wither` variant |
| Forced attack | 3 | Add combat enforcement (similar to Goad) |

**ETB-tapped lands** (12): crypt_of_agadeem, den_of_the_bugbear, gruul_turf, halimar_depths,
indatha_triome, mortuary_mire, oran_rief_the_vastwood, raugrin_triome, savai_triome,
skemfar_elderhall, sparas_headquarters, sunken_palace

**Cycling cards** (5): ziatoras_proving_ground + 4 triomes/headquarters

**New engine variants** (for Wither + forced attack):
- `KeywordAbility::Wither` in `state/types.rs`
- `KeywordAbility::MustAttack` or equivalent in combat enforcement
- Exhaustive match updates: `view_model.rs`, `stack_view.rs`, `replay_harness.rs`

---

### PB-1: Mana With Damage (8 cards, 1 session)

**No engine changes needed.** The DSL already has `Effect::Sequence` and `Effect::DealDamage`.
Pain lands use: `Effect::Sequence(vec![AddMana{...}, DealDamage { target: Controller, amount: 1 }])`.

Exception: City of Brass has a triggered ability ("whenever ~ becomes tapped, it deals 1 damage
to you"). Needs `TriggerCondition::WhenBecomesTapped` — small engine addition.

**Cards**: battlefield_forge, caves_of_koilos, city_of_brass, llanowar_wastes, shivan_reef,
sulfurous_springs, underground_river, yavimaya_coast

**Files**: card_definition.rs (WhenBecomesTapped trigger), card def files (8)

---

### PB-2: Conditional ETB Tapped (56 cards, 2-3 sessions)

Highest card count. Lands enter tapped unless a condition is met.

**Engine change**: Add `unless_condition: Option<Condition>` to `AbilityDefinition::Replacement`.

**New Condition variants needed**:
- `YouControlLandWithSubtype(SubType)` — check-lands (12)
- `YouControlAtMostNOtherLands(u32)` — fast-lands (6)
- `YouHaveTwoOrMoreOpponents` — bond-lands (10)
- `RevealCardOfType(SubType)` — reveal-lands (6)
- `YouControlTwoOrMoreBasicLands` — battle-lands (4)
- `YouControlOtherLandCount { at_most: u32 }` — slow-lands (6+)
- Various castle/misc conditions (12)

**Files**: card_definition.rs (Condition variants), replacement.rs (evaluate condition),
helpers.rs (exports), 56 card def files

---

### PB-3: Shockland ETB (10 cards, 1 session)

"As this enters, you may pay 2 life. If you don't, it enters tapped."

**Engine change**: `ReplacementModification::EntersTappedUnlessPay(Cost)`.
Deterministic fallback: don't pay → enters tapped (conservative, prevents free mana).
Interactive choice deferred to M10.

**Cards**: blood_crypt, breeding_pool, godless_shrine, hallowed_fountain, overgrown_tomb,
sacred_foundry, steam_vents, stomping_ground, temple_garden, watery_grave

**Files**: replacement_effect.rs, replacement.rs, 10 card def files

---

### PB-4: Sacrifice as Activation Cost (26 cards, 2 sessions)

`ActivationCost` has `sacrifice_self: bool` but no filter for "sacrifice a creature."

**Engine change**: Add `sacrifice_filter: Option<TargetFilter>` to `ActivationCost`.
Handle in `command.rs` ability activation.

**Cards**: command_beacon, phyrexian_tower, strip_mine, wasteland, ghost_quarter, high_market,
buried_ruin, scavenger_grounds, etc. (26 total — see audit)

**Files**: game_object.rs (ActivationCost), command.rs, replay_harness.rs (wire sacrifice action),
26 card def files

---

### PB-5: Targeted Activated/Triggered Abilities (32 cards, 2-3 sessions)

**Highest leverage primitive.** `Activated` and `Triggered` lack `targets` fields.

**Engine change**: Add `targets: Vec<TargetRequirement>` to `AbilityDefinition::Activated`
and `AbilityDefinition::Triggered`. Mirror the pattern from `AbilityDefinition::Spell`.
Wire target validation in command.rs (activated) and engine.rs/resolution.rs (triggered).

**Cards**: mother_of_runes, skrelv_defector_mite, yavimaya_hollow, zealous_conscripts,
gilded_drake, reanimate, fell_stinger, access_tunnel, etc. (32 total)

**Files**: card_definition.rs, command.rs, engine.rs, resolution.rs, replay_harness.rs,
32 card def files

---

### PB-6: Static Grant with Controller Filter (30 cards, 1-2 sessions)

"Creatures you control have haste" needs `EffectFilter::CreaturesYouControl`.

**Engine change**: Add `EffectFilter::CreaturesControlledBySource` and
`EffectFilter::CreaturesYouControlWithSubtype(SubType)`. Resolve source controller
at layer-application time in layers.rs.

**Cards**: fervor, mass_hysteria, rhythm_of_the_wild, dragonlord_kolaghan,
goblin_war_drums, brave_the_sands, etc. (30 total)

**Files**: continuous_effect.rs, layers.rs, 30 card def files

---

### PB-7: Count-Based Scaling (29 cards, 2 sessions)

"For each creature you control," "number of lands you control," etc.

**Engine change**: Extend `EffectAmount` with:
- `PermanentCount { filter: TargetFilter, controller: PlayerTarget }`
- `DevotionTo(Color)`
- `CounterCount { target: EffectTarget, counter: CounterType }`

**Cards**: craterhoof_behemoth, gaeas_cradle, nykthos_shrine_to_nyx, cabal_coffers,
blasphemous_act, etc. (29 total)

**Files**: card_definition.rs, effects/mod.rs, 29 card def files

---

### PB-8: Cost Reduction Statics (10 cards, 1-2 sessions)

"Noncreature spells cost {1} more," "Goblin spells cost {1} less."

**Engine change**: `LayerModification::ModifySpellCost { change: i32, filter: SpellCostFilter }`.
Apply in casting.rs at cast time.

**Cards**: thalia_guardian_of_thraben, goblin_warchief, jhoiras_familiar,
danitha_capashen_paragon, the_ur_dragon, etc. (10 total)

**Files**: continuous_effect.rs, casting.rs, card_definition.rs, 10 card def files

---

### PB-9: Hybrid Mana & X Costs (7 cards, 1-2 sessions)

**Engine change**: Add `hybrid: Vec<HybridMana>` and `x_count: u32` to `ManaCost`.
Handle hybrid payment in casting.rs and mana_solver.rs.

**Cards**: brokkos_apex_of_forever, connive, nethroi_apex_of_death, cut_ribbons,
mockingbird, + future X-cost cards

**Files**: game_object.rs (ManaCost), casting.rs, mana_solver.rs, 7 card def files

---

### PB-9.5: Architecture Cleanup (0 cards, 1 session, ~3-4 hours)

No new DSL. Fixes two pieces of accumulated debt that directly reduce the cost of every
subsequent primitive batch. **Do immediately after PB-9.**

#### Fix A: Trigger flush discipline (`engine.rs`, ~1 hour)

The 4-line `check_triggers` + `flush_pending_triggers` pattern is copy-pasted 26+ times
in `process_command`, once per command handler. Some handlers (DeclareAttackers,
DeclareBlockers) skip it — missed invocations silently drop triggers.

**Fix**: Extract the 4-line pattern into a single call at the bottom of `process_command`
after the match, and remove it from every handler arm. One file, mechanical change.

**Risk**: Low. Covered by existing test suite.
**Commit prefix**: `chore:`

#### Fix B: Test file `CardDefinition` defaults (~2-3 hours)

~70 test files construct `CardDefinition` with explicit field enumeration instead of
`..Default::default()`. Every primitive batch that adds a new `CardDefinition` field
requires editing those files — PB-8 touched 15+. This recurs for every future PB that
extends `CardDefinition`.

Card def files (`cards/defs/*.rs`) already use `..Default::default()` — no change needed
there. Only test files need migration.

**Fix**: Find all explicit `CardDefinition { ... }` constructions in `tests/` that lack
`..Default::default()`, add it to each. After this, future PBs only update the
`impl Default` block in `card_definition.rs` — zero test file edits.

**Risk**: Low. Purely additive struct tail; Rust compiler catches any missed cases.
**Commit prefix**: `W6-prim:`

#### Deliberately deferred (post-card-authoring)

These two smells are real but don't affect card authoring velocity:

- **`resolution.rs` split** (7,460 lines → multiple files): Pure organization, no logic
  changes. Safe to do any time. Defer until card authoring completes.
- **`EffectContext` flag accumulation**: Split cast-time flags (`kicker_times_paid`,
  `was_bargained`, etc.) into a `CastMetadata` struct. Growing but not yet blocking.
  Defer until card authoring completes.

---

### PB-10: Return From Zone Effects (8 cards, 1 session)

**Engine change**: Add `TargetRequirement::TargetCardInGraveyard(TargetFilter)`.
Extend targeting system to validate graveyard cards (currently only battlefield/stack).

**Cards**: 8 return-from-zone cards + future recursion/reanimation cards

**Dependencies**: PB-5 (targeting infrastructure)

**Files**: card_definition.rs, command.rs, resolution.rs, 8 card def files

---

### PB-11: Mana Spending Restrictions + ETB Player Choice (13 cards, 2 sessions)

**Engine changes**:
a) `ManaRestriction` enum on `Effect::AddMana` — `CreatureSpellsOnly`, `SubtypeOnly(SubType)`, etc.
   Enforce in casting.rs at mana payment.
b) `chosen_creature_type: Option<SubType>` on `GameObject`. Set via replacement effect on ETB.

**Cards**: cavern_of_souls, secluded_courtyard, unclaimed_territory, + mana-restriction cards (13 total)

**Files**: game_object.rs, casting.rs, card_definition.rs, 13 card def files

---

### PB-12: Complex Replacement Effects (11 cards, 2-3 sessions)

Token/damage/counter doublers and halvers.

**Engine changes**: New `ReplacementModification` variants:
- `DoubleTokens`, `DoubleDamage`, `DoubleCounters`, `HalveCounters`, `CantSearchLibrary`

**Cards**: adrix_and_nev_twincasters, bloodletter_of_aclazotz, vorinclex_monstrous_raider,
aven_mindcensor, pir_imaginative_rascal, tekuthal_inquiry_dominus, etc. (11 total)

**Files**: replacement_effect.rs, replacement.rs, effects/mod.rs, 11 card def files

---

### PB-13: Specialized Mechanics (25+ cards, 5-6 sessions)

| Sub-batch | Primitive | Cards | Effort |
|-----------|-----------|-------|--------|
| 13a | Land animation | 3 | 1 session |
| 13b | Channel ability | 3 | 0.5 session |
| 13c | Ascend / City's Blessing | 2 | 0.5 session |
| 13d | Equipment auto-attach | 2 | 0.5 session |
| 13e | Dredge | 1 | 0.5 session |
| 13f | Buyback (already exists?) | 1 | trivial |
| 13g | Player hexproof | 1 | trivial |
| 13h | Coin flip / d20 | 2 | 1 session |
| 13i | Timing restriction | 2 | 0.5 session |
| 13j | Clone / copy ETB | 2 | 1 session |
| 13k | Monarch designation | 1 | 0.5 session |
| 13l | Flicker (exile + return) | 1+ | 0.5 session |
| 13m | Adventure (split-card from exile) | 1 | 0.5 session |
| 13n | Living weapon (Equipment ETB + Germ token) | 1+ | 0.5 session |

---

## Dependency Graph

```
PB-0 (quick wins) ─── no deps, do first
   │
   ├── PB-1 (mana w/ damage) ─── no deps
   ├── PB-2 (conditional ETB) ─── no deps
   ├── PB-3 (shockland ETB) ─── no deps
   ├── PB-4 (sacrifice cost) ─── no deps
   ├── PB-19 (mass destroy) ─── no deps
   │
   ├── PB-5 (targeted abilities) ─── no deps, HIGH PRIORITY
   │      ├── PB-10 (return from zone) ─── needs PB-5
   │      ├── PB-17 (library search) ─── needs PB-5 targeting
   │      └── PB-21 (fight/bite) ─── needs PB-5 targeting
   │
   ├── PB-6 (static grants) ─── no deps
   │      ├── PB-8 (cost reduction) ─── needs PB-6
   │      └── PB-18 (stax restrictions) ─── needs PB-6 pattern
   │
   ├── PB-7 (count scaling) ─── PB-6 partial
   ├── PB-9 (hybrid/X + phyrexian) ─── no deps
   ├── PB-11 (mana restrict + ETB choice) ─── no deps
   ├── PB-12 (complex replacements) ─── no deps
   ├── PB-13 (specialized + monarch/flicker/adventure) ─── no deps
   ├── PB-14 (planeswalker + emblems, 31+ cards) ─── no deps
   ├── PB-15 (saga/class) ─── no deps
   ├── PB-16 (meld) ─── no deps
   └── PB-20 (additional combat phases) ─── no deps
```

---

## Dangerous Partial Implementations — Immediate Actions

| Category | Cards | Action |
|----------|-------|--------|
| ETB-tapped missing (unconditional) | 12 | **Fix at PB-0** |
| ETB-tapped missing (conditional) | 56 | **Fix at PB-2** |
| ETB-tapped missing (shocklands) | 10 | **Fix at PB-3** |
| Mana without damage | 8 | **Fix at PB-1** |
| Approximated hybrid mana | 5 | **Fix at PB-9** |
| Wrong target types | 2 | **Fix now** (no new DSL needed) |
| Activated abilities missing restrictions | 10 | **Revert to `vec![]`** until primitive built |
| Static effects missing filters | 8 | **Revert to `vec![]`** until PB-6 |
| **Total dangerous** | **~122** | |

---

## Heavy Primitives (formerly "deferrals" — all required pre-alpha)

These are high-effort primitives that require new subsystems, not just new enum variants.
They are **not deferred** — they are scheduled after PB-0 through PB-13.

### PB-14: Planeswalker Support (4 cards, 4-6 sessions)

Full planeswalker framework:
- `CardType::Planeswalker` in type system
- Loyalty counter as a resource (starting loyalty from card def)
- `AbilityDefinition::LoyaltyAbility { cost: i32, effect: Effect }` (+N/-N/-X)
- One loyalty ability per turn restriction
- Damage redirects to planeswalkers (CR 306.7 — player may redirect)
- Planeswalker uniqueness rule (legend rule already exists, extends naturally)
- SBA: planeswalker with 0 loyalty is put into graveyard (CR 704.5i)

**Cards**: ajani_sleeper_agent, tyvar_jubilant_brawler, + 2 others from universe
**Files**: card_definition.rs, game_object.rs, sba.rs, combat.rs, layers.rs, command.rs

### PB-15: Saga & Class Mechanics (2+ cards, 2-3 sessions)

Saga framework:
- Lore counters added on ETB and after draw step
- Chapter abilities trigger when lore counter count reaches chapter number
- Sacrifice after final chapter (SBA)
- `AbilityDefinition::SagaChapter { chapter: u32, effect: Effect }`

Class framework (if cards in universe):
- Level-up activated ability
- Cumulative static abilities per level

**Cards**: urzas_saga, + any Class cards from universe
**Files**: card_definition.rs, game_object.rs, sba.rs, turn_actions.rs

### PB-16: Meld (1 card, 1-2 sessions)

- Meld pairs tracked on CardDefinition (front + paired card)
- Command::Meld checks both cards present on battlefield
- Melded permanent has combined characteristics
- Zone-change splits back into individual cards (similar to Mutate)

**Cards**: hanweir_battlements (melds with Hanweir Garrison)
**Files**: card_definition.rs, game_object.rs, command.rs

### PB-17: Library Search Filters (74 cards, 3-4 sessions)

**The single biggest gap from the unauthored card analysis.** `Effect::SearchLibrary`
exists but only supports `basic_land_filter()`. 74 unauthored cards need to search for
non-basic cards: creatures by type/CMC, artifacts, enchantments, instants/sorceries, etc.

**Engine change**: Extend `SearchFilter` enum with:
- `CreatureCard` / `CreatureWithSubtype(SubType)`
- `ArtifactCard` / `ArtifactWithCmc(u32)`
- `EnchantmentCard`
- `InstantOrSorceryCard`
- `CardWithCmc { cmc: u32, comparison: Ordering }` (e.g., CMC <= 3)
- `CardWithType(CardType)`
- `LandCard` (non-basic land search)
- `AnyCard` (for general tutors like Demonic Tutor)

The interactive choice ("which card do you pick?") uses the existing deterministic fallback
(min ObjectId). Interactive selection deferred to M10.

**Sample cards**: Enlightened Tutor, Mystical Tutor, Green Sun's Zenith, Chord of Calling,
Eladamri's Call, Demonic Tutor, Vampiric Tutor, Worldly Tutor, etc.

**Files**: card_definition.rs (SearchFilter variants), effects/mod.rs (filter matching),
replay_harness.rs (search action), 74 card def files

### PB-18: Stax / Action Restriction Continuous Effects (13 cards, 2 sessions)

"Opponents can't cast spells during your turn," "Players can't cast noncreature spells,"
"Creatures can't attack you unless their controller pays {2}."

**Engine change**: New `ContinuousRestriction` system:
- `Restriction::CantCastSpells { filter: SpellFilter, affected: PlayerTarget }`
- `Restriction::CantAttackYouUnlessPay { cost: ManaCost }` (Propaganda/Ghostly Prison)
- `Restriction::CantActivateAbilities { filter, affected }`
- `Restriction::CantPlayLands { affected }`
- Checked in casting.rs, combat.rs, and command.rs at action-legality time

**Sample cards**: Drannith Magistrate, Propaganda, Ghostly Prison, Silence,
Myrel Shield of Argive, Rule of Law, Archon of Emeria, etc.

**Files**: continuous_effect.rs, casting.rs, combat.rs, legal_actions.rs, 13 card def files

### PB-19: Mass Destroy / Board Wipes (12 cards, 1-2 sessions)

`Effect::DestroyPermanent` targets a single permanent. Board wipes need
`Effect::DestroyAll { filter: TargetFilter }`.

**Engine change**: Add `Effect::DestroyAll { filter: TargetFilter }`. The effect iterates
all permanents matching the filter and destroys them simultaneously (SBA-style batch).
Also add `Effect::ExileAll { filter }` for exile-based wipes.

**Sample cards**: Wrath of God (already exists?), Vanquish the Horde, Fumigate,
Bane of Progress, Austere Command, Ruinous Ultimatum, etc.

**Files**: card_definition.rs, effects/mod.rs, 12 card def files

### PB-20: Additional Combat Phases (10 cards, 2 sessions)

"There is an additional combat phase after this phase" — turn structure must support
inserting combat phases dynamically.

**Engine change**: `Effect::AdditionalCombatPhase`. The turn structure (turn_actions.rs)
needs to support a queue of pending combat phases. When the effect resolves, push a
new combat phase onto the queue. Also need to handle untap-attacking-creatures for
cards like Aurelia ("untap all creatures you control."

**Sample cards**: Aurelia the Warleader, Combat Celebrant, Aggravated Assault,
Moraug Fury of Akoum, Hellkite Charger, etc.

**Files**: card_definition.rs, effects/mod.rs, turn_actions.rs, 10 card def files

### PB-21: Fight & Bite (5+ cards, 1 session)

"Target creature you control fights target creature you don't control."
"This creature deals damage equal to its power to target creature."

**Engine change**:
- `Effect::Fight { attacker: EffectTarget, defender: EffectTarget }` — both deal damage to each other
- `Effect::Bite { source: EffectTarget, target: EffectTarget }` — one-sided damage

**Sample cards**: Infectious Bite, Warstorm Surge, Archdruid's Charm, etc.

**Files**: card_definition.rs, effects/mod.rs, 5+ card def files

---

## Gaps Already Covered by Engine (verify wiring only)

These effects exist in the engine but need verification that card defs can use them.
No new primitives needed — just confirm the wiring works during authoring.

| Pattern | Cards | Engine Support | Action |
|---------|-------|---------------|--------|
| Proliferate | 26 | `Effect::Proliferate` exists | Verify wiring at authoring time |
| Exile-and-play | 17 | `Effect::PlayExiledCard` exists | May need duration param — check |
| Damage each opponent | 16 | `PlayerTarget::EachOpponent` exists | Wire in card defs |
| Emblem creation | 11 | — | Bundle with PB-14 (planeswalker) |
| Phyrexian mana | 7 | — | Bundle with PB-9 (hybrid/X) |

---

## Phase 2: Complete Authoring (~1,025 remaining cards)

After all primitives (PB-0 through PB-16) are implemented, the remaining ~1,025 cards
from the 1,743-card universe can be authored in bulk waves. With all primitives in place,
every card should be fully expressible — no TODOs.

### Authoring Wave Plan

Waves are ordered by group complexity (simplest first, to validate primitives work):

| Wave | Group | Est. Cards | Sessions | Primitive Prerequisites |
|------|-------|-----------|----------|------------------------|
| W-A | Body-only (vanilla/keyword-only) | ~55 | 4 | PB-0 |
| W-B | Mana producers (lands/artifacts/dorks) | ~80 | 6 | PB-1, PB-11 |
| W-C | ETB-tapped lands (remaining) | ~40 | 3 | PB-2, PB-3 |
| W-D | Simple removal (destroy/exile target) | ~50 | 4 | PB-5 |
| W-E | Token creators | ~100 | 8 | PB-5, PB-7 |
| W-F | Anthem/lord creatures | ~60 | 5 | PB-6 |
| W-G | Combat tricks & pump spells | ~50 | 4 | PB-5 |
| W-H | Draw & card advantage | ~80 | 6 | PB-5, PB-7 |
| W-I | Sacrifice outlets & aristocrats | ~50 | 4 | PB-4, PB-5 |
| W-J | Counter manipulation (+1/+1, etc.) | ~40 | 3 | PB-7 |
| W-K | Recursion & reanimation | ~40 | 3 | PB-5, PB-10 |
| W-L | Cost reducers & taxers | ~30 | 2 | PB-8 |
| W-M | Complex spells (modal, X-cost, hybrid) | ~50 | 4 | PB-9 |
| W-N | Replacement effects (doublers, etc.) | ~20 | 2 | PB-12 |
| W-O | Specialty (planeswalkers, sagas, etc.) | ~10 | 2 | PB-14, PB-15, PB-16 |
| W-P | Remaining uncategorized | ~220 | 16 | All PBs |
| **Total** | | **~1,025** | **~76** | |

Each wave follows the same workflow as W5 card authoring:
1. Generate card list from authoring plan universe
2. Run bulk-card-author agents (sessions of ~15 cards)
3. Run card-batch-reviewer agents
4. Fix HIGH/MEDIUM findings
5. Commit

### Wave Execution Strategy

- Run 2-3 authoring sessions in parallel per wave
- Run 4-5 review batches in parallel
- Each wave produces a commit: `W6-cards: Wave W-X — [group] (N cards)`
- Target: all 1,743 cards authored and correct

---

## Phase 3: Final Audit & Certification

After all cards are authored:

1. **Full TODO scan**: `grep -r "TODO" crates/engine/src/cards/defs/` — target: **zero TODOs**
2. **Partial implementation scan**: Check for any remaining wrong-behavior cards
3. **Oracle text verification**: Spot-check 10% of card defs against MCP oracle text lookup
4. **Build + test**: `cargo build --workspace && cargo test --all && cargo clippy -- -D warnings`
5. **Card count verification**: Confirm 1,743 card defs registered in CardRegistry

**Acceptance criteria**: All 1,743 cards have complete, correct implementations. Zero TODOs.
Zero partial implementations. Every card produces correct game state during testing.

---

## Workstream & Commit Convention

**New workstream**: W6 — Primitive Implementation + Complete Card Authoring
**Commit prefixes**:
- `W6-prim:` — primitive batch (engine changes + card fixes)
- `W6-cards:` — bulk authoring wave (card defs only, after primitives)
- `W6-audit:` — final audit fixes

W5 (wave-based card authoring) is **retired**. All card work goes through W6.

---

## Session Workflow

### Per Primitive Batch (PB-N)
1. **Engine implementation**: Add enum variants, execution arms, exhaustive match updates
2. **Unit tests**: Validate primitive works correctly (cite CR)
3. **Card def fixes**: Fix all existing card defs the primitive unblocks
4. **New card defs**: Author new cards from authoring universe now expressible
5. **Review**: Run card-batch-reviewer on changed/new card defs
6. **Build verification**: `cargo build --workspace && cargo test --all && cargo clippy -- -D warnings`
7. **Commit**: `W6-prim: PB-N — implement [primitive], fix/author N cards`

### Per Authoring Wave (W-X)
1. **Generate card list** from authoring plan universe, filtered to group
2. **Run bulk-card-author** agents (2-3 in parallel, ~15 cards each)
3. **Run card-batch-reviewer** agents (4-5 in parallel)
4. **Fix** HIGH/MEDIUM findings
5. **Build verification**: `cargo build --workspace && cargo test --all`
6. **Commit**: `W6-cards: Wave W-X — [group] (N cards)`

---

## Phase 1.5: DSL Gap Closure (PB-23 through PB-37)

Card authoring (Phase 2) revealed that PB-0 through PB-22 left significant DSL gaps.
As of 2026-03-23, 814 of 1,452 card defs (56%) have TODOs. These batches close the
remaining gaps. Full plan with gap inventory: `docs/dsl-gap-closure-plan.md`.

Each batch uses `/implement-primitive`. After each batch, backfill all existing card
defs it unblocks — the batch is not done until all unblocked TODOs are removed.

| Batch | Gap ID | Summary | Cards Unblocked |
|-------|--------|---------|----------------:|
| PB-23 | G-1 | Controller-filtered creature triggers | ~145 |
| PB-24 | G-2 | Conditional statics ("as long as X") | ~201 |
| PB-25 | G-3 | Continuous effect grants | ~98 |
| PB-26 | G-4,9-15 | Trigger variants (all remaining) | ~72 |
| PB-27 | G-5 | X-cost spells | ~42 |
| PB-28 | G-6 | CDA / count-based P/T | ~32 |
| PB-29 | G-7 | Cost reduction statics | ~30 |
| PB-30 | G-8 | Combat damage triggers | ~49 |
| PB-31 | G-16,17 | Cost primitives (RemoveCounter, SacrificeCost) | ~23 |
| PB-32 | G-18-21 | Static/effect (lands, prevention, control, animation) | ~39 |
| PB-33 | G-22,28 | Copy/clone + exile/flicker timing | ~39 |
| PB-34 | G-23-25 | Mana production (filter, devotion, conditional) | ~40 |
| PB-35 | G-27,29,30 | Modal triggers + graveyard + planeswalker | ~60 |
| PB-36 | G-31 | Evasion/protection extensions | ~21 |
| PB-37 | G-26 | Complex activated abilities (residual) | TBD |

### Execution order

1. PB-23, PB-26, PB-30 (trigger gaps — highest leverage, ~266 cards)
2. PB-24, PB-25 (static gaps — ~299 cards)
3. PB-27, PB-28, PB-29 (cost/layer gaps — ~104 cards)
4. PB-31, PB-34, PB-32 (cost + mana + effect primitives — ~102 cards)
5. PB-33, PB-35, PB-36 (complex interactions — ~120 cards)
6. PB-37 (residual — re-assess after 23-36)

### Batch Details

See `docs/dsl-gap-closure-plan.md` for the full gap inventory, engine change
descriptions, and backfill protocol.

---

## Total Effort Estimate

| Phase | Sessions | Cards |
|-------|----------|-------|
| Phase 1: Primitive batches (PB-0 to PB-22) | 42-60 | ~400 fixed + new |
| Phase 1.5: Gap closure (PB-23 to PB-37) | TBD | ~814 backfilled |
| Phase 2: Complete authoring (remaining) | reduced | re-triage after 1.5 |
| Phase 3: Final audit | 2-3 | fixes only |
