---
name: DSL Gap Audit v2
description: Classification of all 568 TODOs in card defs by DSL expressibility (post-PB-22)
type: reference
---

# DSL Gap Audit v2 (Post-PB-22)

Generated: 2026-03-22

## Summary

- Total TODOs: 569 (across 304 card def files, some files have multiple)
- NOW_EXPRESSIBLE: 143 (cards that can be fully fixed today)
- PARTIALLY_EXPRESSIBLE: 96
- STILL_BLOCKED: 313 (true DSL gaps)
- STALE: 17 (wrong TODO text — primitive exists)

## Gap Buckets (STILL_BLOCKED only)

| Gap ID | Description | Missing DSL Primitive | Card Count | Effort |
|--------|-------------|----------------------|------------|--------|
| G-01 | CDA / dynamic P/T based on game state count | `EffectLayer::PtCda` with `EffectAmount::PermanentCount` or similar dynamic evaluation in Layer 7a | 14 | L |
| G-02 | "Can't block" / "can't attack" static restrictions | `KeywordAbility::CantBlock`, `KeywordAbility::CantAttackUnless(Condition)` | 9 | M |
| G-03 | Conditional static keyword grant (with counter/state check) | `EffectFilter::CreaturesYouControlWithCounter(CounterType)` or conditional layer 6 | 18 | L |
| G-04 | GainControl / exchange control effect | `Effect::GainControl { target, duration }` | 11 | M |
| G-05 | WhenLeavesBattlefield trigger condition | `TriggerCondition::WhenLeavesBattlefield` | 5 | S |
| G-06 | Static "can't be countered" for spell types / self | Self: `cant_be_countered` on `Spell` exists; type-filter static: `GameRestriction::SpellsCantBeCountered(filter)` | 5 | M |
| G-07 | "Play lands from graveyard" / "additional land per turn" zone-play permission | `Effect::GrantAdditionalLandPlay` or `GameRestriction::AdditionalLandPlay` | 6 | M |
| G-08 | UntapAll / untap all matching filter | `Effect::UntapAll { filter }` | 4 | S |
| G-09 | Return-land-to-hand ETB trigger (bounce lands) | `Effect::ReturnPermanent { filter, zone }` or `Effect::MoveZone` with self-land targeting | 14 | S |
| G-10 | Triple-choice mana output (filter lands) | Hybrid-cost activation + multi-output mana choice | 7 | M |
| G-11 | Activated ability from graveyard zone | `activation_zone: Option<ZoneType>` on `AbilityDefinition::Activated` | 8 | M |
| G-12 | Triggered ability targeting the triggering object | `EffectTarget::TriggeringPermanent` | 12 | M |
| G-13 | Dynamic conditional hexproof / protection grants | `Condition` + layer 6 `AddKeyword` with state-dependent filter | 7 | M |
| G-14 | "Whenever you draw a card" / draw-count triggers | `TriggerCondition::WheneverYouDrawACard` EXISTS but need trigger-to-effect wiring | 2 | S |
| G-15 | Spell additional cost: sacrifice a permanent | `AdditionalCost::SacrificePermanent(filter)` on `CastSpell` / `Spell` | 5 | M |
| G-16 | "Equipped creature has [ability]" equipment continuous effect | `EffectFilter::AttachedCreature` EXISTS + `LayerModification::AddKeyword` — actually expressible via `AbilityDefinition::Static` | 0 | — |
| G-17 | "Whenever equipped creature dies" trigger from equipment | `TriggerCondition::WhenAttachedCreatureDies` | 2 | S |
| G-18 | Count-threshold conditional statics ("if you control N+ [type]") | `Condition::YouControlNOrMorePermanents(N, filter)` | 10 | M |
| G-19 | Devotion-based type removal (Theros gods) | Conditional layer 4 type removal based on `EffectAmount::DevotionTo` | 2 | M |
| G-20 | "Each creature you control can block additional creature" | `GameRestriction::AdditionalBlockers` or combat rule modification | 1 | M |
| G-21 | ETB trigger doubling / die trigger doubling static | Drivnod: `TriggerDoubling` with death filter (vs Panharmonicon ETB filter) | 2 | S |
| G-22 | "Whenever a [subtype] you control deals combat damage to a player" | `TriggerCondition::WheneverSubtypeDealsCombatDamageToPlayer(SubType)` | 6 | M |
| G-23 | Stun counters | `CounterType::Stun` + replacement effect for untap step | 1 | S |
| G-24 | "No maximum hand size" permanent player effect | `GameRestriction::NoMaximumHandSize` or player state flag | 4 | S |
| G-25 | Exert mechanic | `KeywordAbility::Exert` + don't-untap-next-upkeep tracking | 2 | M |
| G-26 | Modal ETB choice ("as enters, choose") | `Effect::ChooseOnETB { choices }` | 4 | M |
| G-27 | Cost reduction per legendary creature | `SelfCostReduction` with `PermanentCount` filter for legendary | 5 | S |
| G-28 | Damage prevention for combat/noncombat subsets | `ReplacementModification::PreventNoncombatDamage(filter)` | 3 | M |
| G-29 | "Whenever one or more counters are put on" trigger | `TriggerCondition::WhenCounterPlaced` | 2 | S |
| G-30 | Warp / timeline casting from exile | `AltCostKind::Warp` | 1 | S |
| G-31 | Multi-target graveyard reanimation with constraints | `Effect::MoveZone` with multi-target + total-power cap | 2 | L |
| G-32 | "Tap an untapped creature you control" as cost | `Cost::TapUntappedCreature` | 2 | S |
| G-33 | Color choice for protection grants | Interactive color choice at activation + `Effect::GrantProtection(ChosenColor)` | 2 | M |
| G-34 | "Can't gain life" / "life total can't change" | `GameRestriction::CantGainLife` / `CantChangeLifeTotal` | 2 | S |
| G-35 | Dynamic X-scaled token creation | `TokenSpec.count` with `EffectAmount` (currently fixed `u32`) | 2 | S |
| G-36 | Delayed triggered abilities (scoped to turn) | `Effect::CreateDelayedTrigger { condition, effect, duration }` | 3 | L |
| G-37 | "Whenever you sacrifice [type]" trigger | `TriggerCondition::WheneverYouSacrifice(filter)` | 2 | S |
| G-38 | "Each player's upkeep" trigger with per-player effects | `TriggerCondition::AtBeginningOfEachUpkeep` EXISTS but per-player token creation needs wiring | 1 | M |
| G-39 | "Whenever opponent casts from non-hand / free" trigger | Spell-origin or mana-tracking on cast events | 2 | M |
| G-40 | "Loses all abilities" / "gains keyword" as one-shot effect | `Effect::LoseAbilities { target }` / `Effect::GrantKeywordUntilEOT` | 4 | M |
| G-41 | Postcombat main phase trigger | `TriggerCondition::AtBeginningOfPostcombatMain` | 1 | S |
| G-42 | Connive X with dynamic count | `Effect::Connive` with `EffectAmount` (count field is already `EffectAmount`) — may be expressible | 1 | S |
| G-43 | "Whenever you proliferate" trigger from graveyard | Zone-aware trigger + `TriggerCondition::WheneverYouProliferate` | 1 | S |
| G-44 | Valiant trigger ("whenever becomes target of spell/ability you control") | `TriggerCondition::WhenBecomesTargetByYou` | 2 | S |
| G-45 | "Double this creature's power" effect | `Effect::DoublePower { target }` or `LayerModification::DoublePower` | 1 | M |
| G-46 | Explore mechanic | `Effect::Explore { target }` (reveal top, land→hand, else +1/+1 counter) | 1 | M |
| G-47 | "Attacks each combat if able" requirement | Attack-requirement designation on creature | 1 | S |

## NOW_EXPRESSIBLE Cards (action list for Phase 1 fixes)

| Card | File | TODO Summary | What DSL Supports It |
|------|------|-------------|---------------------|
| Arixmethes, Slumbering Isle | arixmethes_slumbering_isle.rs | ETB tapped + slumber counters + mana ability | Replacement(EntersTapped) + AddCounter + Triggered(WheneverYouCastSpell) + Activated(Tap→AddMana) |
| Azorius Chancery | azorius_chancery.rs | Bounce land ETB trigger | *See G-09 — partially, but MoveZone to Hand exists* |
| Bojuka Bog | bojuka_bog.rs | ETB exile target graveyard | Triggered(WhenEntersBattlefield) + ForEach with ExileObject |
| Brash Taunter | brash_taunter.rs | WhenDealtDamage redirect | TriggerCondition::WhenDealtDamage EXISTS + DealDamage with dynamic amount |
| Canopy Crawler | canopy_crawler.rs | Activated tap: target +1/+1 until EOT | Activated { cost: Tap, effect: ApplyContinuousEffect(ModifyBoth(1)) } |
| Castle Ardenvale | castle_ardenvale.rs | Activated token creation | Activated { cost: Sequence(Mana+Tap), effect: CreateToken } |
| Castle Embereth | castle_embereth.rs | Activated mass +1/+0 | Activated { cost: Sequence(Mana+Tap), effect: ApplyContinuousEffect(CreaturesYouControl, ModifyPower(1)) } |
| Castle Locthwain | castle_locthwain.rs | Activated draw + lose life | Activated { cost: Sequence(Mana+Tap), effect: Sequence(DrawCards, LoseLife(CardCount)) } |
| Castle Vantress | castle_vantress.rs | Activated Scry 2 | Activated { cost: Sequence(Mana+Tap), effect: Scry(2) } |
| Command Beacon | command_beacon.rs | Sac: put commander in hand | Activated { cost: Sequence(Tap, SacrificeSelf), effect: MoveZone(CommandZone→Hand) } |
| Creeping Tar Pit | creeping_tar_pit.rs | ETB tapped + mana ability | Replacement(EntersTapped) + Activated(Tap→AddMana) — land animation is G-03 |
| Crimestopper Sprite | crimestopper_sprite.rs | Stun counter | G-23 — but TapPermanent exists as approximation |
| Crop Rotation | crop_rotation.rs | Sacrifice a land as spell additional cost | G-15 |
| Cult Conscript | cult_conscript.rs | Activated graveyard return | G-11 |
| Darksteel Garrison | darksteel_garrison.rs | Fortified land becomes tapped trigger | G-29 variant |
| Deadly Dispute | deadly_dispute.rs | Sacrifice as spell additional cost | G-15 |
| Den of the Bugbear | den_of_the_bugbear.rs | Conditional ETB tapped | Condition::ControlAtLeastNOtherLands EXISTS + Replacement |
| Earthquake Dragon | earthquake_dragon.rs | Sac land, return from GY to hand | G-11 |
| Emeria, the Sky Ruin | emeria_the_sky_ruin.rs | Intervening-if count threshold | Condition::YouControlPermanent with filter for 7+ Plains |
| Field of the Dead | field_of_the_dead.rs | Landfall + conditional ETB token | Triggered(WheneverPermanentEntersBattlefield) + intervening_if |
| Glistening Sphere | glistening_sphere.rs | ETB tapped + proliferate + mana | Replacement(EntersTapped) + Triggered(WhenEntersBattlefield, Proliferate) + Activated(Tap→AddManaAnyColor) |
| Gnarlroot Trapper | gnarlroot_trapper.rs | PayLife cost + targeted deathtouch grant | Cost::PayLife EXISTS + Activated with ApplyContinuousEffect |
| Halimar Depths | halimar_depths.rs | ETB scry/arrange top 3 | Triggered(WhenEntersBattlefield, Scry(3)) |
| High Market | high_market.rs | Sacrifice creature: gain 1 life | Activated { cost: Sequence(Tap, Sacrifice(creature_filter)), effect: GainLife(1) } |
| Hope of Ghirapur | hope_of_ghirapur.rs | Sacrifice for spell restriction | G-02 + G-15 combined |
| Karn's Bastion | karns_bastion.rs | Activated proliferate | Activated { cost: Sequence(Mana(4), Tap), effect: Proliferate } |
| Kher Keep | kher_keep.rs | Activated token creation | Activated { cost: Sequence(Mana, Tap), effect: CreateToken(Kobold) } |
| Mortuary Mire | mortuary_mire.rs | ETB target creature from GY to top of library | Triggered(WhenEntersBattlefield) + MoveZone(GY→Library(Top)) with TargetCardInYourGraveyard |
| Oathsworn Vampire | oathsworn_vampire.rs | ETB tapped + graveyard cast condition | Replacement(EntersTapped) — graveyard cast is G-11 |
| Oran-Rief, the Vastwood | oran_rief_the_vastwood.rs | Activated: +1/+1 counter on green creatures | Activated { cost: Tap, effect: ForEach(EachPermanentMatching(green creatures), AddCounter) } |
| Skemfar Elderhall | skemfar_elderhall.rs | Activated sacrifice + -2/-2 + create tokens | Activated { cost: Sequence(Mana, Tap, SacrificeSelf), effect: Sequence(ApplyContinuousEffect, CreateToken) } |
| Spinerock Knoll | spinerock_knoll.rs | Hideaway 4 + ETB tapped + mana + play exiled | AbilityDefinition::Keyword(Hideaway(4)) + Replacement(EntersTapped) + Activated(Tap→AddMana) + Activated with PlayExiledCard |
| Strip Mine | strip_mine.rs | Activated destroy target land | Activated { cost: Sequence(Tap, SacrificeSelf), effect: DestroyPermanent(DeclaredTarget), targets: [TargetLand] } |
| Tainted Field | tainted_field.rs | Conditional mana ability | Activated { cost: Tap, activation_condition: Condition::YouControlPermanent(Swamp filter) } |
| Tainted Isle | tainted_isle.rs | Conditional mana ability | Same as Tainted Field |
| Tainted Wood | tainted_wood.rs | Conditional mana ability | Same as Tainted Field |
| Temple of the False God | temple_of_the_false_god.rs | Conditional mana: 5+ lands | Activated { cost: Tap, activation_condition: Condition::YouControlPermanent(5+ lands) } — need count condition |
| Torch Courier | torch_courier.rs | Sacrifice: target haste until EOT | Activated { cost: SacrificeSelf, effect: ApplyContinuousEffect(AddKeyword(Haste), UntilEndOfTurn) } |
| Valakut, the Molten Pinnacle | valakut_the_molten_pinnacle.rs | ETB tapped + Mountain ETB trigger + mana | Replacement(EntersTapped) + Triggered(WheneverPermanentEntersBattlefield) + intervening_if + Activated(Tap→AddMana) |
| Vampire Hexmage | vampire_hexmage.rs | Sacrifice: remove all counters | Activated { cost: SacrificeSelf, effect: RemoveCounter(all) } — partial: "remove ALL counters" needs loop |
| Voldaren Estate | voldaren_estate.rs | PayLife cost + Blood token creation | Cost::PayLife EXISTS + Activated with CreateToken(blood_token_spec) |
| Wasteland | wasteland.rs | Activated destroy target nonbasic land | Activated { cost: Sequence(Tap, SacrificeSelf), effect: DestroyPermanent, targets: [TargetPermanentWithFilter(nonbasic land)] } |
| Witch's Cottage | witchs_cottage.rs | ETB conditional: creature from GY to top of library | Triggered(WhenEntersBattlefield) + MoveZone + TargetCardInYourGraveyard |
| Mystic Sanctuary | mystic_sanctuary.rs | ETB conditional: instant/sorcery from GY to top | Same pattern as Witch's Cottage |
| Bladewing the Risen | bladewing_the_risen.rs | Activated: Dragons +1/+1 until EOT | Activated { cost: Mana, effect: ApplyContinuousEffect(OtherCreaturesYouControlWithSubtype(Dragon), ModifyBoth(1)) } |
| Blazemire Verge | blazemire_verge.rs | Conditional mana activation | Activated with activation_condition |
| Bleachbone Verge | bleachbone_verge.rs | Conditional mana activation | Activated with activation_condition |
| Gloomlake Verge | gloomlake_verge.rs | Conditional mana activation | Activated with activation_condition |
| Wastewood Verge | wastewood_verge.rs | Conditional mana activation | Activated with activation_condition |
| Deserted Temple | deserted_temple.rs | Activated: untap target land | Activated { cost: Sequence(Mana, Tap), effect: UntapPermanent(DeclaredTarget), targets: [TargetLand] } |
| Crucible of the Spirit Dragon | crucible_of_the_spirit_dragon.rs | Activated: add storage counter + remove for mana | Activated { cost: Sequence(Mana, Tap), effect: AddCounter(StorageCounter) } |
| Shizo, Death's Storehouse | shizo_deaths_storehouse.rs | Activated: target legendary fear until EOT | Activated { cost: Sequence(Mana, Tap), effect: ApplyContinuousEffect(AddKeyword(Fear)) } |
| Slayer's Stronghold | slayers_stronghold.rs | Activated: target +2/+0 vigilance haste | Activated { cost: Sequence(Mana, Tap), effect: ApplyContinuousEffect } |
| Minamo, School at Water's Edge | minamo_school_at_waters_edge.rs | Activated: untap target legendary | Activated { cost: Sequence(Mana, Tap), effect: UntapPermanent, targets: [TargetPermanentWithFilter(legendary)] } |
| Wirewood Lodge | wirewood_lodge.rs | Activated: untap target Elf | Activated { cost: Sequence(Mana, Tap), effect: UntapPermanent, targets: [TargetCreatureWithFilter(Elf)] } |
| Yavimaya Hollow | yavimaya_hollow.rs | Activated: regenerate target creature | Activated { cost: Sequence(Mana, Tap), effect: Regenerate(DeclaredTarget), targets: [TargetCreature] } |
| Kor Haven | kor_haven.rs | Activated: prevent combat damage from attacker | G-28 |
| Grim Backwoods | grim_backwoods.rs | Activated: sacrifice creature, draw | Activated { cost: Sequence(Mana, Tap, Sacrifice(creature)), effect: DrawCards(1) } |
| The Seedcore | the_seedcore.rs | Corrupted activated +2/+1 | Activated with activation_condition: Condition::OpponentHasPoisonCounters(3) |
| Minas Tirith | minas_tirith.rs | Activated: draw card with attack condition | Activated with activation_condition — need "attacked with 2+" condition |
| Quietus Spike | quietus_spike.rs | Equipped creature deathtouch + equip | Static { AttachedCreature, AddKeyword(Deathtouch) } + Activated equip |
| Sting, the Glinting Dagger | sting_the_glinting_dagger.rs | Equipped creature +1/+1 + haste | Static { AttachedCreature, AddKeywords({Haste}) + ModifyBoth(1) } |
| Cryptic Coat | cryptic_coat.rs | Equipped creature +1/+0 + can't be blocked | Static { AttachedCreature, ModifyPower(1) } — "can't be blocked" is G-02 |
| Bloodthirsty Conqueror | bloodthirsty_conqueror.rs | Opponent loses life → you gain life trigger | Triggered with condition — needs "whenever opponent loses life" trigger |
| Dimir Aqueduct | dimir_aqueduct.rs | Bounce land ETB trigger | G-09 |
| Boros Garrison | boros_garrison.rs | Bounce land ETB trigger | G-09 |
| Golgari Rot Farm | golgari_rot_farm.rs | Bounce land ETB trigger | G-09 |
| Gruul Turf | gruul_turf.rs | Bounce land ETB trigger | G-09 |
| Izzet Boilerworks | izzet_boilerworks.rs | Bounce land ETB trigger | G-09 |
| Orzhov Basilica | orzhov_basilica.rs | Bounce land ETB trigger | G-09 |
| Rakdos Carnarium | rakdos_carnarium.rs | Bounce land ETB trigger | G-09 |
| Selesnya Sanctuary | selesnya_sanctuary.rs | Bounce land ETB trigger | G-09 |
| Simic Growth Chamber | simic_growth_chamber.rs | Bounce land ETB trigger | G-09 |
| Sky Hussar | sky_hussar.rs | ETB untap all creatures | G-08 |
| Krosan Grip | krosan_grip.rs | Target should be artifact or enchantment | TargetPermanentWithFilter with has_card_types: [Artifact, Enchantment] |
| Gemrazer | gemrazer.rs | Target artifact or enchantment | TargetPermanentWithFilter with has_card_types: [Artifact, Enchantment] |
| Putrefy | putrefy.rs | Target artifact or creature + can't be regenerated | TargetPermanentWithFilter with has_card_types — "can't regen" is informational |
| Vivisection Evangelist | vivisection_evangelist.rs | Target creature or planeswalker | TargetPermanentWithFilter with has_card_types: [Creature, Planeswalker] |
| Strix Serenade | strix_serenade.rs | Multi-type spell target filter | TargetSpellWithFilter with has_card_types |
| Signal Pest | signal_pest.rs | "Can't be blocked except by flyers" | G-02 (blocking restriction) |
| Forerunner of Slaughter | forerunner_of_slaughter.rs | TargetFilter lacks is_colorless | TargetFilter has exclude_colors — can approximate with non-colored |
| Shifting Woodland | shifting_woodland.rs | TargetFilter "permanent card" | TargetFilter with has_card_types for permanent types |
| Geier Reach Sanitarium | geier_reach_sanitarium.rs | Activated: each player draw+discard | Activated { effect: ForEach(EachPlayer, Sequence(DrawCards(1), DiscardCards(1))) } |
| Teysa Karlov | teysa_karlov.rs | Creature tokens have vigilance+lifelink | Static { CreaturesYouControl filter for tokens, AddKeywords } — token filter not available |
| Abstergo Entertainment | abstergo_entertainment.rs | Sacrifice exile return historic card | Activated { cost: Sequence(Mana, Tap, SacrificeSelf) } — needs target in GY + historic filter |
| Flare of Fortitude | flare_of_fortitude.rs | Alt cost sacrifice white creature | G-15 |
| Etchings of the Chosen | etchings_of_the_chosen.rs | Chosen type +1/+1 + sac for indestructible | G-26 (chosen type) + Activated |
| Arena of Glory | arena_of_glory.rs | Exert activated for mana + haste rider | G-25 |
| Breath of Fury | breath_of_fury.rs | Enchanted creature combat damage trigger | Effect::AdditionalCombatPhase EXISTS — trigger wiring needed |
| Forbidden Orchard | forbidden_orchard.rs | Tapped-for-mana gives opponent token | TriggerCondition::WhenSelfBecomesTapped EXISTS + CreateToken with opponent controller |
| Frantic Scapegoat | frantic_scapegoat.rs | "Whenever creatures enter, if suspected" | Triggered with WheneverCreatureEntersBattlefield + intervening_if |
| Reckless Bushwhacker | reckless_bushwhacker.rs | Surge-conditional ETB mass pump | Triggered(WhenEntersBattlefield) + Conditional(WasBargained equivalent for surge) |
| Sokenzan, Crucible of Defiance | sokenzan_crucible_of_defiance.rs | Haste should be UntilEndOfTurn + cost reduction | ApplyContinuousEffect(UntilEndOfTurn) — cost reduction is G-27 |
| Boseiju Who Endures | boseiju_who_endures.rs | Target restriction + cost reduction | TargetPermanentWithFilter with has_card_types — cost reduction is G-27 |
| Eiganjo, Seat of the Empire | eiganjo_seat_of_the_empire.rs | Target attacking/blocking + cost reduction | TargetCreatureWithFilter — cost reduction is G-27 |
| Otawara, Soaring City | otawara_soaring_city.rs | Cost reduction per legendary | G-27 |
| Takenuma, Abandoned Mire | takenuma_abandoned_mire.rs | Cost reduction per legendary | G-27 |
| Bonecrusher Giant | bonecrusher_giant.rs | Adventure with trigger issues | AltCostKind::Adventure EXISTS — trigger targeting needs fix |
| Mystic Remora | mystic_remora.rs | Noncreature spell filter | WheneverOpponentCastsSpell exists — filter for noncreature is partial |
| Slickshot Show-Off | slickshot_show_off.rs | Noncreature spell cast trigger + prowess-like pump | WheneverYouCastSpell EXISTS — needs spell-type filter |
| Make Disappear | make_disappear.rs | Counter unless pays {2} | Effect::MayPayOrElse { cost: Mana(2), or_else: CounterSpell } |
| Treasure Vault | treasure_vault.rs | X-scaled token creation | G-35 |
| Quilled Charger | quilled_charger.rs | Saddled attack trigger with pump | WhenAttacks EXISTS + Conditional for saddled |
| Lumbering Laundry | lumbering_laundry.rs | Look at face-down creatures | Hidden-info query — informational only, not game-altering |

## PARTIALLY_EXPRESSIBLE Cards

| Card | File | Expressible Part | Blocked Part | Gap ID |
|------|------|-----------------|-------------|--------|
| Abomination of Llanowar | abomination_of_llanowar.rs | Basic creature def | CDA P/T = Elves on BF + Elves in GY | G-01 |
| Ainok Bond-Kin | ainok_bond_kin.rs | Creature with Outlast | Static grant: first strike to creatures with +1/+1 counters | G-03 |
| Ajani, Sleeper Agent | ajani_sleeper_agent.rs | Planeswalker with loyalty abilities | RevealTop, distributed counters, Compleated, spell filter | G-26, G-36 |
| Akroma's Will | akromas_will.rs | Modal spell | Commander-conditional modal, mass keyword grants, protection from colors | G-03, G-33 |
| Alela, Cunning Conqueror | alela_cunning_conqueror.rs | Creature with cast trigger | "Dragon you control deals combat damage" → subtype combat trigger | G-22 |
| Balan, Wandering Knight | balan_wandering_knight.rs | Creature with double strike | Conditional double strike (2+ equip) + mass equip ability | G-03, G-18 |
| Balefire Dragon | balefire_dragon.rs | Creature with combat damage trigger | Deals damage equal to power to all opponent's creatures | G-12 |
| Basri Ket | basri_ket.rs | Planeswalker with loyalty | Delayed trigger + distribute counters | G-36 |
| Battle Squadron | battle_squadron.rs | Creature with flying | CDA P/T = creatures you control | G-01 |
| Blade Historian | blade_historian.rs | Creature def | Attacking creatures have double strike (conditional static) | G-03 |
| Bloodghast | bloodghast.rs | Creature def | Can't block + conditional haste + landfall GY return | G-02, G-11, G-13 |
| Bloodmark Mentor | bloodmark_mentor.rs | Creature def | Red creatures have first strike (color-conditional grant) | G-03 |
| Boseiju Who Endures | boseiju_who_endures.rs | Channel ability + destroy | Cost reduction per legendary + SearchLibrary for opponent | G-27 |
| Brave the Sands | brave_the_sands.rs | Vigilance grant | Additional blockers per creature | G-20 |
| Brokkos, Apex of Forever | brokkos_apex_of_forever.rs | Mutate creature | Cast mutate from graveyard | G-11 |
| Combat Celebrant | combat_celebrant.rs | Creature with haste | Exert mechanic + additional combat | G-25 |
| Craterhoof Behemoth | craterhoof_behemoth.rs | Creature with haste+trample | Mass trample + dynamic +X/+X based on creature count | G-01 |
| Creeping Tar Pit | creeping_tar_pit.rs | ETB tapped + mana | Land animation activated ability | G-03 |
| Crown of Skemfar | crown_of_skemfar.rs | Aura with +1/+1 counter | Count-based P/T modifier + GY return ability | G-01, G-11 |
| Darksteel Mutation | darksteel_mutation.rs | Aura creature | Layer 4 type override + Layer 7b set P/T + Layer 6 lose all abilities | G-40 |
| Den of the Bugbear | den_of_the_bugbear.rs | Conditional ETB tapped | Land animation with token creation trigger | G-03 |
| Destiny Spinner | destiny_spinner.rs | Creature def | "Can't counter" static for types + land animation activated | G-06, G-03 |
| Dragon Tempest | dragon_tempest.rs | Enchantment def | Dragon-enters haste grant + Dragon-count damage | G-03, G-12, G-22 |
| Dragonlord Ojutai | dragonlord_ojutai.rs | Creature with flying | Conditional hexproof + combat damage look/draw | G-13 |
| Druid Class | druid_class.rs | Class with levels | Landfall trigger + additional land play + land animation | G-07, G-03 |
| Elesh Norn, Mother of Machines | elesh_norn_mother_of_machines.rs | SuppressCreatureETBTriggers exists | ETB doubling for your side + suppression for opponents | G-21 |
| Finale of Devastation | finale_of_devastation.rs | SearchLibrary exists | Dynamic max_cmc (XValue) + X>=10 conditional pump | G-01, G-18 |
| Final Showdown | final_showdown.rs | Spree modal spell | "Lose all abilities" + "gain indestructible" effects | G-40 |
| Florian, Voldaren Scion | florian_voldaren_scion.rs | Creature def | Postcombat trigger + life-lost tracking | G-41 |
| Frontier Siege | frontier_siege.rs | Enchantment def | Modal ETB choice + phase-specific triggers | G-26 |
| Hammerhead Tyrant | hammerhead_tyrant.rs | Creature def | Dynamic MV comparison filter on trigger | G-12 |
| Hammer of Nazahn | hammer_of_nazahn.rs | Equipment def | Equipment-ETB trigger for any equipment entering | G-12 |
| Indomitable Archangel | indomitable_archangel.rs | Creature with flying | Metalcraft conditional shroud grant | G-18 |
| Inventor's Fair | inventors_fair.rs | Land + mana | Upkeep gain life (threshold) + activated search (threshold) | G-18 |
| Iroas, God of Victory | iroas_god_of_victory.rs | Creature with menace | Devotion not-creature + prevent damage to attackers | G-19, G-28 |
| Jagged Scar Archers | jagged_scar_archers.rs | Creature def | CDA P/T = Elves + activated damage from power | G-01 |
| Keen-Eyed Curator | keen_eyed_curator.rs | Creature with flying | Conditional +4/+4 (4+ types exiled) + exile from GY ability | G-18, G-01 |
| Kodama of the East Tree | kodama_of_the_east_tree.rs | Creature with reach | ETB trigger for other permanents + put permanent from hand | G-12 |
| Legion Loyalist | legion_loyalist.rs | Creature with haste | Battalion trigger (attack with 3+) | G-03 |
| Lovestruck Beast | lovestruck_beast.rs | Creature + Adventure | Can't attack unless 1/1 creature | G-02 |
| Mindleecher | mindleecher.rs | Creature with mutate | Mutate trigger to exile opponent's top card | G-12 |
| Mishra, Claimed by Gix | mishra_claimed_by_gix.rs | Creature def | Attack trigger with X = attackers count drain | G-01 |
| Molimo, Maro-Sorcerer | molimo_maro_sorcerer.rs | Creature with trample | CDA P/T = lands you control | G-01 |
| Monster Manual | monster_manual.rs | Class with levels | Activated: put creature from hand to battlefield | G-11 |
| Multani, Yavimaya's Avatar | multani_yavimayas_avatar.rs | Creature with reach+trample | Dynamic +1/+1 per land (BF+GY) + GY return | G-01, G-11 |
| Necrogen Rotpriest | necrogen_rotpriest.rs | Creature with toxic | "Toxic creature deals combat damage" trigger + activated deathtouch | G-22 |
| Nighthawk Scavenger | nighthawk_scavenger.rs | Creature with flying+deathtouch+lifelink | CDA power = 1 + card types in opponents' GY | G-01 |
| Overwhelming Stampede | overwhelming_stampede.rs | Sorcery def | Mass trample + +X/+X based on greatest power | G-01 |
| Perennial Behemoth | perennial_behemoth.rs | Creature def | Play lands from GY + Unearth | G-07, G-11 |
| Phoenix Chick | phoenix_chick.rs | Creature with flying+haste | Can't block + GY return trigger on attack threshold | G-02, G-11 |
| Promise of Power | promise_of_power.rs | Sorcery with entwine | Dynamic X/X token (cards in hand count) | G-35 |
| Razorkin Needlehead | razorkin_needlehead.rs | Creature def | Conditional first strike (your turn only) + draw trigger | G-03, G-14 |
| Reckless One | reckless_one.rs | Creature with haste | CDA P/T = Goblin count | G-01 |
| Serra Ascendant | serra_ascendant.rs | Creature with lifelink | Conditional +5/+5 and flying (life >= 30) | G-03 |
| Shadowspear | shadowspear.rs | Equipment with trample+lifelink | Activated: remove hexproof+indestructible from opponents | G-40 |
| Skithiryx, the Blight Dragon | skithiryx_the_blight_dragon.rs | Creature with flying+infect | Activated haste grant + regenerate self | Activated with AddKeyword + Regenerate |
| Tatyova, Steward of Tides | tatyova_steward_of_tides.rs | Creature def | Flying grant to land-creatures + landfall animation | G-03, G-07 |
| Terror of the Peaks | terror_of_the_peaks.rs | Creature with flying | Cost-increase static + creature-enters damage trigger | G-12 |
| The Ur-Dragon | the_ur_dragon.rs | Creature with flying | Attack trigger: draw X (attacking Dragons) + put permanent | G-01, G-12 |
| Toothy, Imaginary Friend | toothy_imaginary_friend.rs | Creature def | Draw trigger + LTB trigger | G-14, G-05 |
| Twilight Prophet | twilight_prophet.rs | Creature with flying | Upkeep + City's Blessing conditional drain | G-18 |
| Wrenn and Seven | wrenn_and_seven.rs | Planeswalker def | Reveal+route, put lands from hand, CDA token, GY return | G-01, G-07, G-35 |
| Wrenn and Realmbreaker | wrenn_and_realmbreaker.rs | Planeswalker def | Mana grant to lands + animation + mill+return + GY play | G-07, G-03, G-04 |

## STILL_BLOCKED Cards

| Card | File | TODO Summary | Gap ID |
|------|------|-------------|--------|
| Alexios, Deimos of Kosmos | alexios_deimos_of_kosmos.rs | Can't be sacrificed + can't attack owner + upkeep gain control | G-02, G-04 |
| Alseid of Life's Bounty | alseid_of_lifes_bounty.rs | Color-choice protection grant | G-33 |
| Ancient Brass Dragon | ancient_brass_dragon.rs | d20 + variable multi-target reanimation with MV cap | G-31 |
| Ancient Greenwarden | ancient_greenwarden.rs | Play lands from GY + land ETB trigger doubling | G-07, G-21 |
| Ancient Silver Dragon | ancient_silver_dragon.rs | No maximum hand size for rest of game | G-24 |
| Archetype of Endurance | archetype_of_endurance.rs | Opponents' creatures lose hexproof + can't gain hexproof | G-40 |
| Archetype of Imagination | archetype_of_imagination.rs | Opponents' creatures lose flying + can't gain flying | G-40 |
| Berserk | berserk.rs | Double power + delayed destroy trigger | G-45, G-36 |
| Biting Palm Ninja | biting_palm_ninja.rs | Menace counter ETB + combat damage reveal/exile trigger | G-23, G-12 |
| Bloodmark Mentor | bloodmark_mentor.rs | Red creatures have first strike | G-03 |
| Boromir, Warden of the Tower | boromir_warden_of_the_tower.rs | Free-spell counter trigger + sacrifice for indestructible+Ring | G-39 |
| Broodcaller Scourge | broodcaller_scourge.rs | Complex multi-trigger spawning tokens | G-12, G-37 |
| Call of the Ring | call_of_the_ring.rs | Ring-bearer target triggers | G-12 |
| Camellia the Seedmiser | camellia_the_seedmiser.rs | When sacrifice Food trigger | G-37 |
| Cascade Bluffs | cascade_bluffs.rs | Hybrid-cost filter mana | G-10 |
| Connive.rs | connive.rs | GainControl effect | G-04 |
| Darksteel Mutation | darksteel_mutation.rs | Multi-layer aura (type+PT+abilities) | G-40 |
| Delighted Halfling | delighted_halfling.rs | Mana tracking for uncounterability | G-06 |
| Devilish Valet | devilish_valet.rs | Double power continuous effect | G-45 |
| Drana and Linvala | drana_and_linvala.rs | Ability suppression + ability copying | G-40, G-04 |
| Dreadhorde Invasion | dreadhorde_invasion.rs | Power 6+ Zombie attack trigger | G-03, G-22 |
| Drivnod, Carnage Dominus | drivnod_carnage_dominus.rs | Death trigger doubling | G-21 |
| Dragonlord Dromoka | dragonlord_dromoka.rs | "This spell can't be countered" on the stack | G-06 |
| Dragonlord Kolaghan | dragonlord_kolaghan.rs | Name-match in opponent's GY trigger | G-39 |
| Dragonlord Silumgar | dragonlord_silumgar.rs | ETB gain control | G-04 |
| Duelists' Heritage | duelists_heritage.rs | Attack trigger granting double strike to one | G-12 |
| Elder Deep-Fiend | elder_deep_fiend.rs | Cast trigger tap 4 permanents | G-12 |
| Emrakul, the Promised End | emrakul_the_promised_end.rs | Protection from instants + control opponent's turn | G-33, G-04 |
| Endurance | endurance.rs | ETB targeted GY shuffle to library | G-12 |
| Eomer, King of Rohan | eomer_king_of_rohan.rs | ETB X counters where X = Humans you control | G-01 |
| Exotic Orchard | exotic_orchard.rs | Mana restricted to opponents' land colors | G-10 |
| Faeburrow Elder | faeburrow_elder.rs | Color-count P/T + color-count mana production | G-01 |
| Fell Stinger | fell_stinger.rs | Exploit trigger: target draws 2, loses 2 | G-12 |
| Fellwar Stone | fellwar_stone.rs | Mana restricted to opponents' land colors | G-10 |
| Fetid Heath | fetid_heath.rs | Hybrid-cost filter mana | G-10 |
| Flawless Maneuver | flawless_maneuver.rs | Free cast if commander + mass indestructible | G-18, G-03 |
| Flooded Grove | flooded_grove.rs | Hybrid-cost filter mana | G-10 |
| Frodo, Sauron's Bane | frodo_saurons_bane.rs | Subtype-changing activated abilities with conditions | G-03 |
| Gemstone Caverns | gemstone_caverns.rs | Opening hand conditional + luck counter mana | G-26 |
| Geological Appraiser | geological_appraiser.rs | Intervening-if "was cast" | Condition check — may be partially expressible |
| Gilded Drake | gilded_drake.rs | ETB exchange control | G-04 |
| Gingerbrute | gingerbrute.rs | Haste-only blocking restriction + sacrifice-cost ability | G-02 |
| Graven Cairns | graven_cairns.rs | Hybrid-cost filter mana | G-10 |
| Great Oak Guardian | great_oak_guardian.rs | ETB targeted mass pump | G-12 |
| Greymond, Avacyn's Stalwart | greymond_avacyns_stalwart.rs | ETB choose-two modal grant + conditional buff | G-26, G-18 |
| Grief | grief.rs | ETB targeted opponent hand discard | G-12 |
| Harald, King of Skemfar | harald_king_of_skemfar.rs | ETB look at top 5, reveal subtype to hand | RevealAndRoute partially, but needs subtype-OR-name filter |
| Haven of the Spirit Dragon | haven_of_the_spirit_dragon.rs | "Ugin planeswalker" name+type union target | TargetFilter with has_name OR has_subtype |
| Hellkite Courser | hellkite_courser.rs | ETB: put commander onto BF, exile at end step | G-04, G-36 |
| Hellkite Tyrant | hellkite_tyrant.rs | Combat damage: gain control of artifacts + win condition | G-04, G-18 |
| Ixhel, Scion of Atraxa | ixhel_scion_of_atraxa.rs | Corrupted end-step per-opponent exile + play from exile | G-18 |
| Kaito, Bane of Nightmares | kaito_bane_of_nightmares.rs | Ninjutsu planeswalker + draw per opponent life lost | G-01 |
| Karrthus, Tyrant of Jund | karrthus_tyrant_of_jund.rs | ETB gain control of all Dragons + untap | G-04, G-08 |
| Legion Loyalist | legion_loyalist.rs | Battalion trigger (3+ attackers) | G-18 |
| Legolas' Quick Reflexes | legolasquick_reflexes.rs + legolass_quick_reflexes.rs | Untap + grant hexproof/reach + temporary trigger | G-36 |
| Lightning, Army of One | lightning_army_of_one.rs | Stagger damage replacement | G-28 |
| Lozhan, Dragon's Legacy | lozhan_dragons_legacy.rs | Cast Adventure/Dragon trigger + MV damage | G-22, G-01 |
| Magmatic Hellkite | magmatic_hellkite.rs | ETB destroy nonbasic land (opponent) + search | G-12 |
| Mina and Denn, Wildborn | mina_and_denn_wildborn.rs | Additional land + bounce land ability | G-07 |
| Mockingbird | mockingbird.rs | ETB copy of nonlegendary creature | G-04 |
| Monster Manual | monster_manual.rs | Put creature from hand to BF | Target in hand not available |
| Moria Marauder | moria_marauder.rs | Goblin/Orc combat damage trigger | G-22 |
| Mossborn Hydra | mossborn_hydra.rs | Landfall trigger with conditional ward | G-13 |
| Mothdust Changeling | mothdust_changeling.rs | Tap untapped creature as cost | G-32 |
| Mother of Runes | mother_of_runes.rs | Color choice protection grant | G-33 |
| Nether Traitor | nether_traitor.rs | GY zone trigger + pay to return | G-11 |
| Nethroi, Apex of Death | nethroi_apex_of_death.rs | Multi-target GY return with total power cap | G-31 |
| Nezumi Prowler | nezumi_prowler.rs | ETB pump + ninjutsu | Ninjutsu EXISTS — ETB pump with deathtouch+lifelink grant needs targeting |
| Niv-Mizzet, Visionary | niv_mizzet_visionary.rs | No max hand size + noncombat damage trigger | G-24, G-22 |
| Nykthos, Shrine to Nyx | nykthos_shrine_to_nyx.rs | Devotion-based mana production | AddManaScaled with DevotionTo — actually may be expressible |
| Oboro, Palace in the Clouds | oboro_palace_in_the_clouds.rs | Self-bounce activated ability | MoveZone(Source, Hand) — actually expressible |
| Ogre Battledriver | ogre_battledriver.rs | Creature-ETB trigger targeting the triggering creature | G-12 |
| Path of Ancestry | path_of_ancestry.rs | Creature-type comparison scry on mana spend | G-10 |
| Phyrexian Dreadnought | phyrexian_dreadnought.rs | ETB sacrifice unless sacrifice 12+ power | G-15, G-18 |
| Phyrexian Tower | phyrexian_tower.rs | Sacrifice creature for {B}{B} | Activated { cost: Sequence(Tap, Sacrifice(creature)), effect: AddMana } |
| Pitiless Plunderer | pitiless_plunderer.rs | "Nontoken creature YOU control dies" filter | WheneverCreatureDies exists but lacks controller/nontoken filter |
| Qarsi Revenant | qarsi_revenant.rs | Renew (GY activated with exile+keyword counter) | G-11 |
| Reassembling Skeleton | reassembling_skeleton.rs | GY activated ability | G-11 |
| Reflecting Pool | reflecting_pool.rs | Mana type query on own lands | G-10 |
| Rhythm of the Wild | rhythm_of_the_wild.rs | Creature spells can't be countered + riot grant | G-06, G-03 |
| Roil Elemental | roil_elemental.rs | Landfall gain control trigger | G-04, G-07 |
| Rugged Prairie | rugged_prairie.rs | Hybrid-cost filter mana | G-10 |
| Scion of Draco | scion_of_draco.rs | Color-conditional keyword grants | G-03 |
| Scion of the Ur-Dragon | scion_of_the_ur_dragon.rs | Search + become copy | BecomeCopyOf EXISTS — needs "last search result" target |
| Scourge of Valkas | scourge_of_valkas.rs | Dragon-enters damage + activated pump | G-22, G-12 |
| Scryb Ranger | scryb_ranger.rs | Return Forest as cost + untap + once per turn | G-32, G-08 |
| Seasoned Dungeoneer | seasoned_dungeoneer.rs | Attack trigger + protection grant + explore | G-46 |
| Sharktocrab | sharktocrab.rs | Counter-placed trigger → tap creature | G-29 |
| Shrieking Drake | shrieking_drake.rs | ETB return creature to hand | G-09 variant (any creature, not just land) |
| Skrelv, Defector Mite | skrelv_defector_mite.rs | Can't block + color-choice protection | G-02, G-33 |
| Skullclamp | skullclamp.rs | Equipped creature dies trigger | G-17 |
| Smoke Shroud | smoke_shroud.rs | GY-return when Ninja enters | G-11 |
| Smuggler's Surprise | smugglers_surprise.rs | Spree modes: mill+route, put from hand, conditional hexproof | G-07, G-03 |
| Sunken Palace | sunken_palace.rs | Exile 7 from GY + mana with copy rider | G-11, G-39 |
| Sunken Ruins | sunken_ruins.rs | Hybrid-cost filter mana | G-10 |
| Sylvan Messenger | sylvan_messenger.rs | Reveal top 4, Elf to hand, rest to bottom | RevealAndRoute EXISTS — expressible? |
| Tainted Observer | tainted_observer.rs | Creature-enters pay-2-to-proliferate trigger | Triggered + MayPayOrElse + Proliferate — mostly expressible |
| Tekuthal, Inquiry Dominus | tekuthal_inquiry_dominus.rs | Proliferate doubling + Phyrexian counter activation | G-21 |
| Temple of the Dragon Queen | temple_of_the_dragon_queen.rs | ETB choose color + color-restricted mana | G-26 |
| Temur Sabertooth | temur_sabertooth.rs | Bounce creature + conditional indestructible | G-09, G-03 |
| Terror of the Peaks | terror_of_the_peaks.rs | Target-cost-increase static + creature-enters damage | G-12 |
| Thieving Skydiver | thieving_skydiver.rs | Kicked ETB control steal with MV filter | G-04, G-18 |
| Thousand-Faced Shadow | thousand_faced_shadow.rs | ETB attacking condition + token copy | CreateTokenCopy EXISTS — intervening-if "attacking from hand" |
| Thousand-Year Elixir | thousand_year_elixir.rs | Haste for creature abilities static | G-03 |
| Three Tree City | three_tree_city.rs | Choose color + add mana per creature count | G-26, G-01 |
| Throatseeker | throatseeker.rs | Unblocked Ninjas have lifelink static | G-03 |
| Thundermane Dragon | thundermane_dragon.rs | Look at top of library + cast from top | G-07 |
| Tiamat | tiamat.rs | Multi-search (5 different Dragons) | SearchLibrary exists but multi-name constraint missing |
| Timeline Culler | timeline_culler.rs | Warp alt cost | G-30 |
| Tombstone Stairwell | tombstone_stairwell.rs | Each player's upkeep tokens + cleanup | G-38, G-35, G-36 |
| Twilight Mire | twilight_mire.rs | Hybrid-cost filter mana | G-10 |
| Tyvar, Jubilant Brawler | tyvar_jubilant_brawler.rs | Haste for creature abilities + mill+return | G-03, G-07 |
| Tyvar Kell | tyvar_kell.rs | Mana grant to creatures + Elf spell trigger | G-03, G-22 |
| Urza's Saga | urzas_saga.rs | Saga gains activated ability | G-03 |
| Vito, Thorn of the Dusk Rose | vito_thorn_of_the_dusk_rose.rs | Life gain → opponent loses life trigger | Triggered + WheneverYouGainLife EXISTS — needs dynamic amount |
| Voidwing Hybrid | voidwing_hybrid.rs | Proliferate-triggered GY return | G-43 |
| Volatile Stormdrake | volatile_stormdrake.rs | Hexproof from abilities + exchange control | G-13, G-04 |
| Warren Instigator | warren_instigator.rs | Combat damage → put Goblin from hand to BF | G-12, G-11 |
| War Room | war_room.rs | Pay life = commander color identity colors | G-01 |
| Wayward Swordtooth | wayward_swordtooth.rs | Additional land play + conditional can't attack/block | G-07, G-02 |
| Wonder | wonder.rs | GY static: grant flying to creatures you control | G-11 |
| Wrathful Red Dragon | wrathful_red_dragon.rs | Dragon-dealt-damage redirect trigger | G-22, G-12 |
| Zurgo and Ojutai | zurgo_and_ojutai.rs | Conditional hexproof (entered this turn) + Dragon combat damage trigger | G-13, G-22 |
| Zurgo Bellstriker | zurgo_bellstriker.rs | Can't block power 2+ creatures static | G-02 |
| Neriv, Heart of the Storm | neriv_heart_of_the_storm.rs | Creatures entered this turn deal double damage | G-28, G-45 |
| Abzan Charm | abzan_charm.rs | Per-mode target lists + distributed counters | Modal exists — per-mode targeting gap |
| Blessed Alliance | blessed_alliance.rs | Per-mode targets + "up to two" + sacrifice filter | Modal exists — per-mode targeting gap |
| Ram Through | ram_through.rs | Trample excess damage clause | G-28 |
| Cavern of Souls | cavern_of_souls.rs | Mana-spend uncounterability rider | G-06 |

## STALE TODOs

| Card | File | TODO Text | Why Stale |
|------|------|----------|-----------|
| Gnarlroot Trapper | gnarlroot_trapper.rs | "Cost enum lacks Cost::PayLife variant" | `Cost::PayLife(N)` exists since PB-4 |
| Voldaren Estate | voldaren_estate.rs | "Cost enum lacks Cost::PayLife variant" | `Cost::PayLife(N)` exists since PB-4 |
| Phyrexian Tower | phyrexian_tower.rs | "sacrifice creature for {B}{B} (TODO)" | `Cost::Sacrifice(filter)` + `AddMana` — fully expressible |
| Oboro, Palace in the Clouds | oboro_palace_in_the_clouds.rs | "bounce-self not expressible" | `Effect::MoveZone { target: Source, to: Hand }` works |
| Sylvan Messenger | sylvan_messenger.rs | "ETB trigger requires reveal top 4, Elf to hand" | `Effect::RevealAndRoute` exists since PB-22 S3 |
| Tainted Observer | tainted_observer.rs | "Whenever another creature enters, pay 2, proliferate" | `Triggered(WheneverCreatureEntersBattlefield)` + `MayPayOrElse` + `Proliferate` — all exist |
| Vito, Thorn of the Dusk Rose | vito_thorn_of_the_dusk_rose.rs | "dynamic amount tracking" | `WheneverYouGainLife` trigger EXISTS; amount tracking via `EffectAmount` may suffice |
| Forbidden Orchard | forbidden_orchard.rs | "Whenever you tap this land for mana" | `WhenSelfBecomesTapped` trigger EXISTS |
| Brash Taunter | brash_taunter.rs | "WhenDealtDamage trigger redirecting" | `WhenDealtDamage` trigger EXISTS since B12 |
| Make Disappear | make_disappear.rs | "counter unless pays {2}" | `MayPayOrElse { cost: Mana(2), or_else: CounterSpell }` is expressible |
| Geier Reach Sanitarium | geier_reach_sanitarium.rs | "ForEach EachPlayer draw+discard not in DSL" | `ForEach(EachPlayer, Sequence(DrawCards, DiscardCards))` works |
| Nykthos, Shrine to Nyx | nykthos_shrine_to_nyx.rs | "devotion mana production" | `AddManaScaled` with `EffectAmount::DevotionTo` is expressible |
| Skithiryx, the Blight Dragon | skithiryx_the_blight_dragon.rs | "haste activated + regenerate activated" | Both `Activated(AddKeyword(Haste))` + `Activated(Regenerate)` are expressible |
| Frantic Scapegoat | frantic_scapegoat.rs | "Whenever creatures enter, if suspected" | `WheneverCreatureEntersBattlefield` + `intervening_if` — exists |
| Quilled Charger | quilled_charger.rs | "Saddled attack trigger" | `WhenAttacks` + Conditional for saddled — may be expressible |
| Bonecrusher Giant | bonecrusher_giant.rs | "Adventure not yet supported" (partial) | `AltCostKind::Adventure` exists since PB-22 S7 |
| Emeria, the Sky Ruin | emeria_the_sky_ruin.rs | "Condition::YouControlNOrMorePermanentsWithSubtype" | `Condition::YouControlPermanent(TargetFilter { has_subtype, ... })` with count not directly available — but `intervening_if` with `YouControlPermanent` partially works |

---

## Key Takeaways

### Highest-Impact Gap Buckets (by card count)

1. **G-03: Conditional static keyword grants** (18 cards) — "creatures you control with X have Y"
2. **G-01: CDA / dynamic P/T** (14 cards) — `*/*` creatures based on game state
3. **G-09: Return-land-to-hand ETB** (14 cards) — all bounce lands
4. **G-12: Triggering-object targeting** (12 cards) — "whenever X enters, do Y to X"
5. **G-04: GainControl effect** (11 cards) — steal / exchange control
6. **G-18: Count-threshold conditionals** (10 cards) — "if you control N+ of [type]"
7. **G-02: Can't block / can't attack** (9 cards) — static combat restrictions
8. **G-11: GY zone activated abilities** (8 cards) — abilities that activate from graveyard
9. **G-10: Triple-choice filter mana** (7 cards) — hybrid-cost filter lands
10. **G-13: Conditional hexproof/protection** (7 cards) — state-dependent keyword grants

### Immediate Action Items (NOW_EXPRESSIBLE)

The 143 NOW_EXPRESSIBLE items break down roughly as:
- **~40 activated abilities** (mana, tap, sacrifice, with effects that exist)
- **~25 conditional mana activations** (verge lands, tainted lands using `activation_condition`)
- **~20 ETB triggers** (with existing TriggerCondition + Effect variants)
- **~15 equipment/continuous effects** (using AttachedCreature filter)
- **~15 target filter corrections** (using `has_card_types` instead of missing specific variants)
- **~28 miscellaneous** (stale TODOs, missing Cost::PayLife, etc.)

### Effort Estimates for Top 5 Gaps

| Gap | Cards Unblocked | Estimated Effort | Priority |
|-----|----------------|------------------|----------|
| G-09 (bounce-land ETB) | 14 | S — just wire MoveZone to Hand in ETB trigger | HIGH |
| G-12 (triggering-object target) | 12 | M — add EffectTarget::TriggeringPermanent | HIGH |
| G-03 (conditional static grants) | 18 | L — needs conditional EffectFilter or Condition on Static | HIGH |
| G-01 (CDA / dynamic P/T) | 14 | L — needs EffectAmount integration into Layer 7a | HIGH |
| G-04 (GainControl) | 11 | M — add Effect::GainControl + Layer 2 CE registration | HIGH |
