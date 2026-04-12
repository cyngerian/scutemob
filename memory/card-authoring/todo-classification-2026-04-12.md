# TODO Classification Report — Card Definition Primitive Categories

**Generated**: 2026-04-12
**Source**: `crates/engine/src/cards/defs/*.rs`

## Summary

- **Total cards with TODOs**: 780
- **Total TODOs**: 1236 (some cards have multiple)
- **Distinct primitive categories**: 190
- **Categorized TODOs**: 1032
- **Uncategorized TODOs**: 204

### Yield Calibration Notes

Per `memory/feedback_pb_yield_calibration.md`, PB planners overcount in-scope cards by 2-3x. Apply 40-50% attrition to yield estimates:
- **Clean pattern-matches** (TargetFilter fields, EffectAmount variants): ~60-70% yield
- **Trigger conditions** (subtype-filtered, conditional): ~50% yield — filtering often reveals additional gaps
- **Complex patterns** (conditional, modal, reflexive): ~30-40% yield — these tend to have compound blockers
- **Interactive/M10+**: 0% yield until M10 — these are deferred by design

### Potentially Stale TODOs (may be fixable now)

These reference primitives that have shipped or partially shipped:

| Card | TODO References | May be fixed by |
|------|-----------------|-----------------|
| song_of_freyalise | "grant via PB-S LayerModification::AddManaAbility" | PB-S (verify) |
| bootleggers_stash | "Lands you control gain activated ability" | PB-S (verify) |
| throne_of_eldraine | "ChosenColor designation" | PB-Q (parked — gauntlet blocked) |

**Action**: Before next PB, grep for these cards and check if their TODOs are now stale.

### Top 20 Categories by Card Count (these drive PB priority)

| Rank | Primitive Category | Cards | TODOs | Existing PB |
|------|-------------------|-------|-------|-------------|
| 1 | DSL Gap (unclassified) | 135 | 147 | new |
| 2 | ComplexPattern::ConditionalIf | 37 | 41 | new |
| 3 | Cost::SacrificeFilteredType | 23 | 30 | new |
| 4 | TargetFilter::UpToN | 21 | 22 | PB-T |
| 5 | Interactive::PlayerChoice | 21 | 24 | M10+ |
| 6 | EffectAmount::CounterCount | 20 | 21 | new |
| 7 | TriggerCondition::SubtypeFilteredAttack | 18 | 18 | new |
| 8 | TargetFilter::DamagedPlayer | 15 | 18 | new |
| 9 | TriggerCondition::SubtypeFilteredDeath | 15 | 16 | new |
| 10 | EffectAmount::PowerOfCreature | 13 | 13 | new |
| 11 | TriggerCondition::Landfall | 13 | 15 | new |
| 12 | TriggerCondition::WheneverYouCastSpell | 11 | 14 | new |
| 13 | TargetFilter::Nontoken | 11 | 15 | PB-23 (partial) |
| 14 | TriggerCondition::WhenAttacks | 11 | 11 | new |
| 15 | TriggerCondition::WheneverOpponentLosesLife | 11 | 13 | new |
| 16 | Cost::TapNCreatures | 10 | 12 | new |
| 17 | Cost::PayLife | 10 | 12 | new |
| 18 | Cost::SacrificeAnother | 10 | 13 | new |
| 19 | EffectAmount::XValue | 9 | 10 | partial — XValue exists |
| 20 | Effect::GrantKeyword | 9 | 10 | new |

---

## Categories (sorted by card count descending)

### 1. DSL Gap (unclassified) — 135 cards

- **TODOs**: 147
- **Estimated LOC**: ~1400
- **Existing PB**: new
- **Cards**: academy_manufactor, al_bhed_salvagers, ancient_brass_dragon, badgermole_cub, benefactors_draught, berserk, betor_kin_to_all, biting_palm_ninja, black_market, black_market_connections, blackblade_reforged, boggart_shenanigans, bonecrusher_giant, boromir_warden_of_the_tower, broodcaller_scourge... (+120 more)
- **Sample TODO**: "Token replacement effect — "instead create one of each" not in DSL"

### 2. ComplexPattern::ConditionalIf — 37 cards

- **TODOs**: 41
- **Estimated LOC**: ~420
- **Existing PB**: new
- **Cards**: ajani_sleeper_agent, archdruids_charm, balan_wandering_knight, betor_kin_to_all, blasphemous_edict, bloodchief_ascension, chart_a_course, cloudstone_curio, dawns_truce, delighted_halfling, dwynen_s_elite, fabled_passage, garruks_uprising, greymond_avacyns_stalwart, guardian_project... (+22 more)
- **Sample TODO**: "RevealTop + conditional hand/library placement."

### 3. Cost::SacrificeFilteredType — 23 cards

- **TODOs**: 30
- **Estimated LOC**: ~280
- **Existing PB**: new
- **Cards**: accursed_marauder, anowon_the_ruin_sage, blasphemous_edict, blessed_alliance, constant_mists, demolition_field, demons_disciple, falkenrath_pit_fighter, flare_of_malice, fleshbag_marauder, grim_hireling, kellogg_dangerous_mind, merciless_executioner, myriad_landscape, professional_face_breaker... (+8 more)
- **Sample TODO**: "SacrificePermanents has no nontoken filter — each player sacrifices any permanent,"

### 4. TargetFilter::UpToN — 21 cards

- **TODOs**: 22
- **Estimated LOC**: ~260
- **Existing PB**: PB-T
- **Cards**: abstergo_entertainment, blessed_alliance, bridgeworks_battle, buried_alive, cloud_of_faeries, elder_deep_fiend, force_of_vigor, frantic_search, glissa_sunslayer, marang_river_regent, mindbreak_trap, skullsnatcher, smugglers_surprise, snap, sorin_lord_of_innistrad... (+6 more)
- **Sample TODO**: "{3}, {T}, Exile Abstergo Entertainment: Return up to one target historic card"

### 5. Interactive::PlayerChoice — 21 cards

- **TODOs**: 24
- **Estimated LOC**: ~260
- **Existing PB**: M10+
- **Cards**: abundance, brainsurge, by_invitation_only, cultivator_colossus, dig_through_time, eerie_ultimatum, experimental_augury, hidden_strings, incendiary_command, mother_of_runes, open_the_vaults, retraced_image, sarkhan_unbroken, saskia_the_unyielding, sorin_imperious_bloodlord... (+6 more)
- **Sample TODO**: "Draw replacement effect with player choice (land/nonland), reveal-until-found,"

### 6. EffectAmount::CounterCount — 20 cards

- **TODOs**: 21
- **Estimated LOC**: ~250
- **Existing PB**: new
- **Cards**: allosaurus_shepherd, anim_pakal_thousandth_moon, arixmethes_slumbering_isle, armorcraft_judge, basri_ket, biting_palm_ninja, cavern_of_souls, crimestopper_sprite, eomer_king_of_rohan, exuberant_fuseling, forgotten_ancient, inspiring_call, mana_leak, mana_tithe, out_of_the_tombs... (+5 more)
- **Sample TODO**: ""Green spells you control can't be countered" — static anti-counter"

### 7. TriggerCondition::SubtypeFilteredAttack — 18 cards

- **TODOs**: 18
- **Estimated LOC**: ~230
- **Existing PB**: new
- **Cards**: aqueous_form, argentum_armor, battle_cry_goblin, bear_umbra, diamond_pick_axe, dreadhorde_invasion, dromoka_the_eternal, hellrider, hermes_overseer_of_elpis, kazuul_tyrant_of_the_cliffs, kolaghan_the_storms_fury, najeela_the_blade_blossom, quilled_charger, sanctum_seeker, shared_animosity... (+3 more)
- **Sample TODO**: ""Whenever enchanted creature attacks, scry 1" — needs a trigger condition"

### 8. TargetFilter::DamagedPlayer — 15 cards

- **TODOs**: 18
- **Estimated LOC**: ~200
- **Existing PB**: new
- **Cards**: alela_cunning_conqueror, balefire_dragon, blood_seeker, cavern_hoard_dragon, horn_of_greed, ink_eyes_servant_of_oni, marisi_breaker_of_the_coil, mistblade_shinobi, mystic_remora, natures_will, polymorphists_jest, skullsnatcher, smothering_tithe, throat_slitter, walker_of_secret_ways
- **Sample TODO**: "Goad effect needs DamagedPlayer resolution for "that player" targeting."

### 9. TriggerCondition::SubtypeFilteredDeath — 15 cards

- **TODOs**: 16
- **Estimated LOC**: ~200
- **Existing PB**: new
- **Cards**: anafenza_unyielding_lineage, athreos_god_of_passage, blade_of_the_bloodchief, crossway_troublemakers, luminous_broodmoth, marionette_apprentice, miara_thorn_of_the_glade, morbid_opportunist, omnath_locus_of_rage, pashalik_mons, patron_of_the_vein, serpents_soul_jar, skullclamp, teysa_orzhov_scion, thornbite_staff
- **Sample TODO**: ""Whenever another nontoken creature you control dies" — WhenDies with"

### 10. EffectAmount::PowerOfCreature — 13 cards

- **TODOs**: 13
- **Estimated LOC**: ~180
- **Existing PB**: new
- **Cards**: altar_of_dementia, carmen_cruel_skymarcher, conclave_mentor, elendas_hierophant, greater_good, jagged_scar_archers, juri_master_of_the_revue, krenko_tin_street_kingpin, master_biomancer, nighthawk_scavenger, ruthless_technomancer, the_great_henge, warstorm_surge
- **Sample TODO**: "DSL gap — EffectAmount::PowerOfSacrificedCreature does not exist. The mill amount"

### 11. TriggerCondition::Landfall — 13 cards

- **TODOs**: 15
- **Estimated LOC**: ~180
- **Existing PB**: new
- **Cards**: bojuka_bog, druid_class, field_of_the_dead, khalni_heart_expedition, moraug_fury_of_akoum, mossborn_hydra, mystic_sanctuary, omnath_locus_of_creation, omnath_locus_of_rage, roil_elemental, springheart_nantuko, tatyova_steward_of_tides, witchs_cottage
- **Sample TODO**: "Triggered — When this land enters, exile target player's graveyard."

### 12. TriggerCondition::WheneverYouCastSpell — 11 cards

- **TODOs**: 14
- **Estimated LOC**: ~160
- **Existing PB**: new
- **Cards**: aetherflux_reservoir, arixmethes_slumbering_isle, edgar_markov, emrakul_the_promised_end, esper_sentinel, hammerhead_tyrant, hazorets_monument, lozhan_dragons_legacy, prossh_skyraider_of_kher, ramos_dragon_engine, up_the_beanstalk
- **Sample TODO**: "TriggerCondition::ControllerCastsSpell exists but "gain 1 life for"

### 13. TargetFilter::Nontoken — 11 cards

- **TODOs**: 15
- **Estimated LOC**: ~160
- **Existing PB**: PB-23 (partial)
- **Cards**: ashaya_soul_of_the_wild, flare_of_cultivation, flare_of_denial, flare_of_fortitude, guardian_project, lena_selfless_champion, miirym_sentinel_wyrm, prowess_of_the_fair, rhythm_of_the_wild, skyclave_apparition, the_great_henge
- **Sample TODO**: ""nontoken" filter — EffectFilter::CreaturesYouControl includes tokens."

### 14. TriggerCondition::WhenAttacks — 11 cards

- **TODOs**: 11
- **Estimated LOC**: ~160
- **Existing PB**: new
- **Cards**: aurelia_the_law_above, breena_the_demagogue, drakuseth_maw_of_flames, dwynen_gilt_leaf_daen, edgar_markov, etali_primal_storm, otharri_suns_glory, six, sun_titan, ureni_of_the_unwritten, uro_titan_of_natures_wrath
- **Sample TODO**: ""Whenever a player attacks with 3+ creatures" trigger not in DSL."

### 15. TriggerCondition::WheneverOpponentLosesLife — 11 cards

- **TODOs**: 13
- **Estimated LOC**: ~160
- **Existing PB**: new
- **Cards**: bloodthirsty_conqueror, bolass_citadel, dragonlord_kolaghan, elderfang_venom, exquisite_blood, exsanguinate, massacre_wurm, mindcrank, scrawling_crawler, shaman_of_the_pack, vito_thorn_of_the_dusk_rose
- **Sample TODO**: "Triggered ability — whenever an opponent loses life, you gain that much life."

### 16. Cost::TapNCreatures — 10 cards

- **TODOs**: 12
- **Estimated LOC**: ~150
- **Existing PB**: new
- **Cards**: azami_lady_of_scrolls, blood_tribute, glare_of_subdual, honor_worn_shaku, kyren_negotiations, mothdust_changeling, opposition, patriars_seal, sky_hussar, springleaf_drum
- **Sample TODO**: "Tap an untapped Wizard you control: Draw a card."

### 17. Cost::PayLife — 10 cards

- **TODOs**: 12
- **Estimated LOC**: ~150
- **Existing PB**: new
- **Cards**: call_of_the_ring, crossway_troublemakers, fire_covenant, mystic_forge, ripples_of_undeath, street_wraith, tainted_observer, timeline_culler, toxic_deluge, war_room
- **Sample TODO**: ""Whenever you choose a creature as your Ring-bearer, you may pay 2 life."

### 18. Cost::SacrificeAnother — 10 cards

- **TODOs**: 13
- **Estimated LOC**: ~150
- **Existing PB**: new
- **Cards**: commissar_severina_raine, izoni_thousand_eyed, korvold_fae_cursed_king, prossh_skyraider_of_kher, ruthless_lawbringer, ruthless_technomancer, vampire_gourmand, wight_of_the_reliquary, yawgmoth_thran_physician, ziatora_the_incinerator
- **Sample TODO**: ""Sacrifice another creature" — Cost::SacrificeOther not in DSL."

### 19. EffectAmount::XValue — 9 cards

- **TODOs**: 10
- **Estimated LOC**: ~140
- **Existing PB**: partial — XValue exists
- **Cards**: agadeems_awakening, chord_of_calling, exsanguinate, finale_of_devastation, fire_covenant, green_suns_zenith, ruthless_technomancer, steel_hellkite, ugin_the_spirit_dragon
- **Sample TODO**: ""Return creature cards from your graveyard with different mana values X or less.""

### 20. Effect::GrantKeyword — 9 cards

- **TODOs**: 10
- **Estimated LOC**: ~140
- **Existing PB**: new
- **Cards**: ainok_strike_leader, akromas_will, boromir_warden_of_the_tower, boros_charm, craterhoof_behemoth, final_showdown, inspiring_call, lazotep_plating, smugglers_surprise
- **Sample TODO**: ""Sacrifice this creature: Creature tokens you control gain indestructible"

### 21. EffectAmount::CountCreaturesYouControl — 9 cards

- **TODOs**: 9
- **Estimated LOC**: ~140
- **Existing PB**: new
- **Cards**: ashaya_soul_of_the_wild, dawnstrike_vanguard, eomer_king_of_rohan, harvest_season, master_biomancer, shaman_of_the_pack, throne_of_the_god_pharaoh, ulvenwald_hydra, wrenn_and_seven
- **Sample TODO**: "CDA — P/T equals the number of lands you control. DSL has no CountLandsCDA."

### 22. EffectAmount::CountSubtypeYouControl — 9 cards

- **TODOs**: 11
- **Estimated LOC**: ~140
- **Existing PB**: new
- **Cards**: captivating_vampire, crown_of_skemfar, elven_ambush, elvish_promenade, general_kreat_the_boltbringer, krenko_mob_boss, lathril_blade_of_the_elves, mavren_fein_dusk_apostle, tyvar_the_bellicose
- **Sample TODO**: "DSL gap — "Tap five untapped Vampires you control: Gain control of target"

### 23. Static::ProtectionFromColor — 8 cards

- **TODOs**: 8
- **Estimated LOC**: ~130
- **Existing PB**: new
- **Cards**: alseid_of_lifes_bounty, autumns_veil, commanders_plate, emrakul_the_promised_end, seasoned_dungeoneer, sword_of_body_and_mind, teferis_protection, volatile_stormdrake
- **Sample TODO**: "DSL gap — the activated ability requires "protection from the color of your"

### 24. EffectAmount::CountLandsYouControl — 8 cards

- **TODOs**: 9
- **Estimated LOC**: ~130
- **Existing PB**: new
- **Cards**: avenger_of_zendikar, bootleggers_stash, garruk_primal_hunter, leyline_of_the_guildpact, multani_yavimayas_avatar, scapeshift, spelunking, the_world_tree
- **Sample TODO**: "EffectAmount lacks "count of lands you control" variant."

### 25. TriggerCondition::OncePerTurn — 8 cards

- **TODOs**: 10
- **Estimated LOC**: ~130
- **Existing PB**: new
- **Cards**: baron_bertram_graywater, dusk_legion_duelist, elvish_warmaster, kishla_skimmer, scourge_of_the_throne, scryb_ranger, welcoming_vampire, whispering_wizard
- **Sample TODO**: ""Whenever tokens enter" trigger + "once each turn" not in DSL."

### 26. TriggerCondition::WhenDealsCombatDamage — 8 cards

- **TODOs**: 10
- **Estimated LOC**: ~130
- **Existing PB**: new
- **Cards**: bident_of_thassa, breath_of_fury, dokuchi_silencer, hellkite_tyrant, moria_marauder, necrogen_rotpriest, saskia_the_unyielding, umezawas_jitte
- **Sample TODO**: ""Whenever a creature you control deals combat damage to a player" — this is a"

### 27. EffectAmount::PoisonCounters — 8 cards

- **TODOs**: 9
- **Estimated LOC**: ~130
- **Existing PB**: new
- **Cards**: ichor_rats, ixhel_scion_of_atraxa, phyresis_outbreak, phyrexian_swarmlord, prologue_to_phyresis, skrevls_hive, vishgraz_the_doomhive, vraska_betrayals_sting
- **Sample TODO**: "DSL gap — no Effect variant to give poison counters to players directly."

### 28. TriggerCondition::WhenBecomesTarget — 7 cards

- **TODOs**: 9
- **Estimated LOC**: ~120
- **Existing PB**: new
- **Cards**: bonecrusher_giant, flowerfoot_swordmaster, goldspan_dragon, scalelord_reckoner, scion_of_the_ur_dragon, tectonic_giant, venerated_rotpriest
- **Sample TODO**: "(1): Trigger condition WhenBecomesTargetByOpponent is WRONG — should be"

### 29. ComplexPattern::ReflexiveTrigger — 7 cards

- **TODOs**: 8
- **Estimated LOC**: ~120
- **Existing PB**: new
- **Cards**: brokers_hideout, caesar_legions_emperor, dokuchi_silencer, maestros_theater, mana_vault, ruthless_lawbringer, temur_sabertooth
- **Sample TODO**: "Reflexive trigger pattern ("When you do, ...") is not expressible in the current DSL."

### 30. EffectAmount::HandSize — 7 cards

- **TODOs**: 7
- **Estimated LOC**: ~120
- **Existing PB**: new
- **Cards**: chandra_flamecaller, jeskas_will, promise_of_power, reforge_the_soul, satoru_umezawa, shattered_perception, tectonic_reformation
- **Sample TODO**: ""Discard all cards then draw that many plus one" — EffectAmount::HandSize"

### 31. Effect::ReturnFromGraveyard — 7 cards

- **TODOs**: 8
- **Estimated LOC**: ~120
- **Existing PB**: new
- **Cards**: footbottom_feast, gravepurge, multani_yavimayas_avatar, nether_traitor, nethroi_apex_of_death, return_upon_the_tide, wrenn_and_seven
- **Sample TODO**: "Multi-target graveyard-to-library not expressible. Draw only."

### 32. TriggerCondition::WheneverOpponentDiscards — 7 cards

- **TODOs**: 7
- **Estimated LOC**: ~120
- **Existing PB**: new
- **Cards**: grief, lilianas_caress, megrim, raiders_wake, tinybones_trinket_thief, torment_of_hailfire, waste_not
- **Sample TODO**: "ETB trigger targeting an opponent to reveal their hand and discard a"

### 33. TargetFilter::PerModeTargets — 6 cards

- **TODOs**: 6
- **Estimated LOC**: ~110
- **Existing PB**: new
- **Cards**: abzan_charm, archmages_charm, blessed_alliance, cryptic_command, insatiable_avarice, naya_charm
- **Sample TODO**: "per-mode target lists are not supported; all targets are declared up front."

### 34. Effect::CounterTargetSpell — 6 cards

- **TODOs**: 6
- **Estimated LOC**: ~110
- **Existing PB**: new
- **Cards**: access_denied, keep_safe, pact_of_negation, siren_stormtamer, tibalts_trickery, transcendent_dragon
- **Sample TODO**: "Counter target spell + create X 1/1 Thopter tokens where X = MV."

### 35. TargetFilter::HasCounters — 6 cards

- **TODOs**: 6
- **Estimated LOC**: ~110
- **Existing PB**: new
- **Cards**: ainok_bond_kin, chasm_skulker, dragonstorm_globe, inspiring_call, joraga_warcaller, vampire_socialite
- **Sample TODO**: "Layer 6 static grant — "Each creature you control with a +1/+1 counter"

### 36. ComplexPattern::ModalETB — 6 cards

- **TODOs**: 7
- **Estimated LOC**: ~110
- **Existing PB**: new
- **Cards**: akromas_will, frontier_siege, glacierwood_siege, grenzo_havoc_raiser, greymond_avacyns_stalwart, windcrag_siege
- **Sample TODO**: "DSL gap — conditional modal choice ("if you control a commander, may choose both"),"

### 37. TriggerCondition::EachPlayersUpkeep — 6 cards

- **TODOs**: 9
- **Estimated LOC**: ~110
- **Existing PB**: new
- **Cards**: alexios_deimos_of_kosmos, ruthless_winnower, sheoldred_whispering_one, sting_the_glinting_dagger, tombstone_stairwell, tymna_the_weaver
- **Sample TODO**: "Upkeep trigger (each player's upkeep): GainControl + untap + AddCounters + haste"

### 38. Effect::PutOnTopOfLibrary — 6 cards

- **TODOs**: 7
- **Estimated LOC**: ~110
- **Existing PB**: new
- **Cards**: brainsurge, forever_young, goblin_recruiter, memory_lapse, senseis_divining_top, teferi_hero_of_dominaria
- **Sample TODO**: ""put two cards from hand on top of library" — interactive card selection"

### 39. Static::GrantTriggeredAbility — 6 cards

- **TODOs**: 6
- **Estimated LOC**: ~110
- **Existing PB**: new
- **Cards**: carnelian_orb_of_dragonkind, clavileno_first_of_the_blessed, dionus_elvish_archdruid, legolasquick_reflexes, mirage_phalanx, tyvar_the_bellicose
- **Sample TODO**: "Mana-spend Dragon trigger (haste grant) — no mana-spend trigger in DSL."

### 40. TargetFilter::AttackingOrBlocking — 6 cards

- **TODOs**: 7
- **Estimated LOC**: ~110
- **Existing PB**: new
- **Cards**: commissar_severina_raine, dolmen_gate, eiganjo_seat_of_the_empire, iroas_god_of_victory, najeela_the_blade_blossom, reconnaissance
- **Sample TODO**: ""Each opponent loses X where X = other attacking creatures" —"

### 41. TriggerCondition::WhenTokenEntersOrLeaves — 6 cards

- **TODOs**: 7
- **Estimated LOC**: ~110
- **Existing PB**: new
- **Cards**: curiosity_crafter, kaito_dancing_shadow, lathliss_dragon_queen, mardu_ascendancy, nadiers_nightblade, zurgo_stormrender
- **Sample TODO**: ""Creature token deals combat damage" trigger not in DSL."

### 42. Cost::ExileSelfFromGraveyard — 6 cards

- **TODOs**: 6
- **Estimated LOC**: ~110
- **Existing PB**: new
- **Cards**: drivnod_carnage_dominus, keen_eyed_curator, laelia_the_blade_reforged, qarsi_revenant, scavenging_ooze, sunken_palace
- **Sample TODO**: "Activated ability — {B/P}{B/P}, exile three creature cards from your graveyard:"

### 43. Cost::ExileFromHand — 6 cards

- **TODOs**: 9
- **Estimated LOC**: ~110
- **Existing PB**: new
- **Cards**: elvish_spirit_guide, force_of_despair, force_of_negation, force_of_vigor, force_of_will, simian_spirit_guide
- **Sample TODO**: "Cost::ExileFromHand does not exist. The ability should be activatable"

### 44. Effect::CreateTokenForEach — 5 cards

- **TODOs**: 6
- **Estimated LOC**: ~100
- **Existing PB**: new
- **Cards**: adeline_resplendent_cathar, curse_of_opulence, mahadi_emporium_master, resculpt, tombstone_stairwell
- **Sample TODO**: "Attack trigger creates tokens per-opponent — DSL lacks per-target token"

### 45. Effect::NoMaxHandSize — 5 cards

- **TODOs**: 6
- **Estimated LOC**: ~100
- **Existing PB**: new
- **Cards**: ancient_silver_dragon, curiosity_crafter, nezahal_primal_tide, niv_mizzet_visionary, wrenn_and_seven
- **Sample TODO**: "DSL gap — "no maximum hand size for the rest of the game" requires"

### 46. ComplexPattern::DelayedTrigger — 5 cards

- **TODOs**: 6
- **Estimated LOC**: ~100
- **Existing PB**: new
- **Cards**: basri_ket, brokers_ascendancy, mirage_phalanx, summoners_pact, thaumatic_compass
- **Sample TODO**: "Delayed triggered abilities (scoped to the current turn) are not yet"

### 47. Static::MustAttack — 5 cards

- **TODOs**: 5
- **Estimated LOC**: ~100
- **Existing PB**: new
- **Cards**: bident_of_thassa, goblin_rabblemaster, howlsquad_heavy, legion_warboss, toski_bearer_of_secrets
- **Sample TODO**: ""Creatures your opponents control attack this turn if able" — forced attack"

### 48. Effect::RemoveCounters — 5 cards

- **TODOs**: 5
- **Estimated LOC**: ~100
- **Existing PB**: new
- **Cards**: biting_palm_ninja, crucible_of_the_spirit_dragon, spark_double, strixhaven_stadium, tekuthal_inquiry_dominus
- **Sample TODO**: "triggered — combat damage to player → may remove menace counter → reveal hand, exile nonland card."

### 49. Static::CDABasedOnCount — 5 cards

- **TODOs**: 7
- **Estimated LOC**: ~100
- **Existing PB**: PB-X (partial)
- **Cards**: blackblade_reforged, empyrial_plate, moraug_fury_of_akoum, storm_kiln_artist, wight_of_the_reliquary
- **Sample TODO**: "DSL gap — dynamic +1/+1 per land you control. LayerModification"

### 50. TargetFilter::TypeUnion — 5 cards

- **TODOs**: 7
- **Estimated LOC**: ~100
- **Existing PB**: new
- **Cards**: boseiju_who_endures, kayas_ghostform, mind_games, rith_liberated_primeval, tear_asunder
- **Sample TODO**: "Target filter should restrict to "artifact, enchantment, or nonbasic land" —"

### 51. Static::BlockingRestriction — 5 cards

- **TODOs**: 5
- **Estimated LOC**: ~100
- **Existing PB**: new
- **Cards**: champion_of_lambholt, delney_streetwise_lookout, kaito_shizuki, sundering_eruption, zurgo_bellstriker
- **Sample TODO**: "DSL gap — "Creatures with power less than ~'s power can't block creatures"

### 52. TargetFilter::SubtypeExclusion — 5 cards

- **TODOs**: 5
- **Estimated LOC**: ~100
- **Existing PB**: new
- **Cards**: cult_conscript, elvish_dreadlord, galadhrim_ambush, keeper_of_fables, ruthless_winnower
- **Sample TODO**: ""Activate only if a non-Skeleton creature died under your control"

### 53. Static::CantBeCountered — 5 cards

- **TODOs**: 5
- **Estimated LOC**: ~100
- **Existing PB**: new
- **Cards**: destiny_spinner, mistrise_village, rhythm_of_the_wild, veil_of_summer, vexing_shusher
- **Sample TODO**: "DSL gap — "can't be countered" static for specific spell types not expressible"

### 54. Interactive::TopNSelection — 5 cards

- **TODOs**: 6
- **Estimated LOC**: ~100
- **Existing PB**: new
- **Cards**: dig_through_time, dragonlord_ojutai, satoru_umezawa, stock_up, write_into_being
- **Sample TODO**: ""look at top 7, choose 2" — approximated as DrawCards(2)."

### 55. TriggerCondition::SubtypeFilteredETB — 5 cards

- **TODOs**: 5
- **Estimated LOC**: ~100
- **Existing PB**: new
- **Cards**: ganax_astral_hunter, hammer_of_nazahn, scourge_of_valkas, thornbite_staff, wolverine_riders
- **Sample TODO**: ""Whenever Ganax or another Dragon enters" — subtype-filtered ETB trigger not in DSL"

### 56. Replacement::DrawReplacement — 4 cards

- **TODOs**: 6
- **Estimated LOC**: ~90
- **Existing PB**: new
- **Cards**: abundance, laboratory_maniac, out_of_the_tombs, teferis_ageless_insight
- **Sample TODO**: "Draw replacement effect too complex for DSL."

### 57. TargetFilter::TargetOpponent — 4 cards

- **TODOs**: 5
- **Estimated LOC**: ~90
- **Existing PB**: new
- **Cards**: acererak_the_archlich, ajani_sleeper_agent, brash_taunter, veil_of_summer
- **Sample TODO**: "(M10+): tokens should be created under each opponent's control."

### 58. TriggerCondition::Raid — 4 cards

- **TODOs**: 4
- **Estimated LOC**: ~90
- **Existing PB**: new
- **Cards**: alesha_who_laughs_at_fate, bloodsoaked_champion, raiders_wake, searslicer_goblin
- **Sample TODO**: "DSL gap — Raid: end step trigger with "if you attacked this turn""

### 59. Static::CantAttack — 4 cards

- **TODOs**: 5
- **Estimated LOC**: ~90
- **Existing PB**: new
- **Cards**: alexios_deimos_of_kosmos, lovestruck_beast, the_eternal_wanderer, wayward_swordtooth
- **Sample TODO**: ""can't attack its owner" — attack restriction by owner not in DSL"

### 60. EffectAmount::D20Roll — 4 cards

- **TODOs**: 4
- **Estimated LOC**: ~90
- **Existing PB**: new
- **Cards**: ancient_brass_dragon, ancient_bronze_dragon, ancient_copper_dragon, ancient_gold_dragon
- **Sample TODO**: "DSL gap — d20 roll + variable multi-target reanimation with cumulative MV"

### 61. Effect::TokenDoublingReplacement — 4 cards

- **TODOs**: 4
- **Estimated LOC**: ~90
- **Existing PB**: new
- **Cards**: anointed_procession, doubling_season, parallel_lives, xorn
- **Sample TODO**: "Token doubling replacement effect not in DSL"

### 62. Effect::TapTargetPermanent — 4 cards

- **TODOs**: 4
- **Estimated LOC**: ~90
- **Existing PB**: new
- **Cards**: blasting_station, charismatic_conqueror, nullmage_shepherd, tyvar_kell
- **Sample TODO**: "DSL gap — there is no effect variant for untapping a specific permanent (self), and"

### 63. TriggerCondition::WhenBecomesTapped — 4 cards

- **TODOs**: 4
- **Estimated LOC**: ~90
- **Existing PB**: new
- **Cards**: charismatic_conqueror, legolass_quick_reflexes, mesmeric_orb, quest_for_renewal
- **Sample TODO**: ""Whenever an artifact or creature an opponent controls enters untapped" —"

### 64. Effect::DiscardThenDraw — 4 cards

- **TODOs**: 4
- **Estimated LOC**: ~90
- **Existing PB**: new
- **Cards**: chart_a_course, roiling_dragonstorm, teferi_master_of_time, the_locust_god
- **Sample TODO**: ""Then discard unless you attacked" not expressible."

### 65. EffectAmount::GraveyardCardCount — 4 cards

- **TODOs**: 4
- **Estimated LOC**: ~90
- **Existing PB**: new
- **Cards**: elvish_reclaimer, hallowed_spiritkeeper, izoni_thousand_eyed, six
- **Sample TODO**: ""gets +2/+2 as long as there are three or more land cards in your graveyard""

### 66. EffectAmount::AttackingCreatureCount — 4 cards

- **TODOs**: 5
- **Estimated LOC**: ~90
- **Existing PB**: new
- **Cards**: galadhrim_ambush, grand_warlord_radha, keep_watch, mishra_claimed_by_gix
- **Sample TODO**: ""Create X tokens where X = number of attacking creatures" — EffectAmount"

### 67. Effect::PutFromHandOntoBattlefield — 4 cards

- **TODOs**: 4
- **Estimated LOC**: ~90
- **Existing PB**: new
- **Cards**: goblin_lackey, monster_manual, tooth_and_nail, ugin_the_spirit_dragon
- **Sample TODO**: ""put a Goblin from hand onto battlefield" — needs MoveZone from"

### 68. Static::GrantActivatedAbility — 4 cards

- **TODOs**: 6
- **Estimated LOC**: ~90
- **Existing PB**: new
- **Cards**: lena_selfless_champion, mother_of_runes, necrogen_rotpriest, urzas_saga
- **Sample TODO**: "Sacrifice activated ability grants indestructible only to creatures with"

### 69. Static::CantCastPhaseScoped — 4 cards

- **TODOs**: 5
- **Estimated LOC**: ~90
- **Existing PB**: new
- **Cards**: marisi_breaker_of_the_coil, myrel_shield_of_argive, silence, voice_of_victory
- **Sample TODO**: ""Your opponents can't cast spells during combat" — phase-scoped CantCast not in DSL."

### 70. TriggerCondition::WheneverYouDrawCard — 3 cards

- **TODOs**: 4
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: alandra_sky_dreamer, teferi_hero_of_dominaria, teferi_temporal_pilgrim
- **Sample TODO**: ""Whenever you draw your second card each turn" — requires a per-turn"

### 71. EffectAmount::DevotionCount — 3 cards

- **TODOs**: 5
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: aspect_of_hydra, nykthos_shrine_to_nyx, thassas_oracle
- **Sample TODO**: "CR 702.5c: devotion to green = number of {G} symbols in mana costs of"

### 72. TriggerCondition::Battalion — 3 cards

- **TODOs**: 3
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: aurelia_the_law_above, legion_loyalist, minas_tirith
- **Sample TODO**: ""Whenever a player attacks with 5+ creatures" trigger not in DSL."

### 73. EffectAmount::TriggeringAmount — 3 cards

- **TODOs**: 4
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: balefire_dragon, sanguine_bond, vito_thorn_of_the_dusk_rose
- **Sample TODO**: ""it deals that much damage to each creature that player controls" —"

### 74. Effect::UntapAll — 3 cards

- **TODOs**: 4
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: benefactors_draught, seedborn_muse, sky_hussar
- **Sample TODO**: ""Untap all creatures" — Effect::UntapAll not in DSL."

### 75. AdditionalCost::DiscardCard — 3 cards

- **TODOs**: 3
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: big_score, thrill_of_possibility, tormenting_voice
- **Sample TODO**: "Discard additional cost not expressible. Draw + treasure."

### 76. EffectAmount::HalfLife — 3 cards

- **TODOs**: 3
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: blood_tribute, quietus_spike, vonas_hunger
- **Sample TODO**: ""loses half life rounded up" needs EffectAmount::HalfLife."

### 77. Effect::GainLifeEqualTo — 3 cards

- **TODOs**: 3
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: blood_tribute, diamond_valley, miren_the_moaning_well
- **Sample TODO**: ""if kicked, gain life equal to life lost" needs conditional."

### 78. ComplexPattern::PerOpponentEffect — 3 cards

- **TODOs**: 3
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: braids_arisen_nightmare, ixhel_scion_of_atraxa, press_the_enemy
- **Sample TODO**: "Full ability requires sacrifice-choice, type-matching, per-opponent"

### 79. EffectAmount::GreatestPower — 3 cards

- **TODOs**: 3
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: crackling_doom, garruk_primal_hunter, return_of_the_wildspeaker
- **Sample TODO**: "Second part — "Each opponent sacrifices a creature with the greatest power""

### 80. Static::AuraGrants — 3 cards

- **TODOs**: 3
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: darksteel_mutation, kenriths_transformation, springheart_nantuko
- **Sample TODO**: "DSL gap — this Aura applies multiple simultaneous layer effects to the enchanted creature:"

### 81. EffectAmount::ToughnessOfCreature — 3 cards

- **TODOs**: 3
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: diamond_valley, last_march_of_the_ents, miren_the_moaning_well
- **Sample TODO**: ""gain life equal to the sacrificed creature's toughness" requires"

### 82. TriggerCondition::WhenLeavesBattlefield — 3 cards

- **TODOs**: 5
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: dour_port_mage, sengir_autocrat, tombstone_stairwell
- **Sample TODO**: ""Leave without dying" trigger not in DSL."

### 83. Keyword::LevelUp — 3 cards

- **TODOs**: 4
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: druid_class, innkeepers_talent, joraga_treespeaker
- **Sample TODO**: "Level 3 needs land-animation continuous effect + ETB trigger on level-up"

### 84. Token::TargetController — 3 cards

- **TODOs**: 3
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: emergency_eject, forbidden_orchard, stroke_of_midnight
- **Sample TODO**: "Create a Lander token for the target's controller."

### 85. TriggerCondition::WhenCounterPlaced — 3 cards

- **TODOs**: 3
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: fathom_mage, sharktocrab, simic_ascendancy
- **Sample TODO**: ""Whenever a +1/+1 counter is put on" trigger not in DSL."

### 86. LayerModification::RemoveAbilities — 3 cards

- **TODOs**: 3
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: final_showdown, kenriths_transformation, vraska_betrayals_sting
- **Sample TODO**: "no Effect::LoseAbilities variant exists in the DSL."

### 87. Effect::CounterUnlessPay — 3 cards

- **TODOs**: 3
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: flusterstorm, make_disappear, spell_pierce
- **Sample TODO**: ""unless its controller pays {1}" — CounterUnlessPay not in DSL."

### 88. Effect::DestroyAll — 3 cards

- **TODOs**: 4
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: force_of_despair, martial_coup, steel_hellkite
- **Sample TODO**: "DSL gap — "destroy all creatures that entered this turn" requires a TargetFilter that"

### 89. Replacement::LeylineOpening — 3 cards

- **TODOs**: 4
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: gemstone_caverns, leyline_of_abundance, leyline_of_the_guildpact
- **Sample TODO**: ""If this card is in your opening hand and you're not the starting player" ETB"

### 90. Static::CantUntap — 3 cards

- **TODOs**: 4
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: goblin_sharpshooter, hands_of_binding, mana_vault
- **Sample TODO**: ""doesn't untap during your untap step" — static restriction not in DSL."

### 91. Effect::ImpulseDraw — 3 cards

- **TODOs**: 3
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: golos_tireless_pilgrim, jeskas_will, ragavan_nimble_pilferer
- **Sample TODO**: "{2}{W}{U}{B}{R}{G}: Exile top 3 cards, you may play them this turn without paying mana costs"

### 92. Keyword::Ninjutsu — 3 cards

- **TODOs**: 5
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: kaito_bane_of_nightmares, satoru_umezawa, silver_fur_master
- **Sample TODO**: "Ninjutsu cost activated ability — ninjutsu is handled by the keyword"

### 93. Effect::PhaseOut — 3 cards

- **TODOs**: 4
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: kaito_shizuki, teferi_master_of_time, teferis_protection
- **Sample TODO**: ""If entered this turn, phases out" — conditional end-step phase-out."

### 94. TargetFilter::Nonlegendary — 3 cards

- **TODOs**: 3
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: kiki_jiki_mirror_breaker, persist, woodland_bellower
- **Sample TODO**: "target filter lacks "nonlegendary" restriction (TargetFilter has no nonlegendary bool)."

### 95. ComplexPattern::ETBCopyEffect — 3 cards

- **TODOs**: 3
- **Estimated LOC**: ~80
- **Existing PB**: new
- **Cards**: mockingbird, phantasmal_image, sakashimas_student
- **Sample TODO**: "DSL gap — the ETB copy effect requires:"

### 96. Keyword::Exert — 2 cards

- **TODOs**: 2
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: arena_of_glory, combat_celebrant
- **Sample TODO**: "Activated — {R}, {T}, Exert this land: Add {R}{R}. If that mana is spent on a creature spell, it gai..."

### 97. Static::GlobalETBReplacement — 2 cards

- **TODOs**: 2
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: biting_palm_ninja, everflowing_chalice
- **Sample TODO**: "ETB — enters with a menace counter."

### 98. TargetFilter::TappedUntapped — 2 cards

- **TODOs**: 2
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: blind_obedience, charismatic_conqueror
- **Sample TODO**: ""Artifacts and creatures your opponents control enter tapped" — needs a global"

### 99. TargetFilter::SharesType — 2 cards

- **TODOs**: 2
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: cloudstone_curio, myriad_landscape
- **Sample TODO**: ""return another permanent that shares a permanent type with it" — shared-type check"

### 100. ComplexPattern::CommandZoneInteraction — 2 cards

- **TODOs**: 3
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: command_beacon, hellkite_courser
- **Sample TODO**: "{T}, Sacrifice: Put your commander into your hand from the command zone"

### 101. EffectAmount::CountArtifactsYouControl — 2 cards

- **TODOs**: 4
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: dispatch, inventors_fair
- **Sample TODO**: "Metalcraft conditional — if 3+ artifacts, exile the same target creature instead"

### 102. TriggerCondition::CombatDamageSubtyped — 2 cards

- **TODOs**: 2
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: dragonlord_ojutai, yuriko_the_tigers_shadow
- **Sample TODO**: ""Whenever Dragonlord Ojutai deals combat damage to a player, look at the top three"

### 103. Replacement::ETBWithCounters — 2 cards

- **TODOs**: 2
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: dragonstorm_globe, volatile_stormdrake
- **Sample TODO**: "ETB replacement for Dragons (+1/+1 counter)"

### 104. Static::AbilitiesOfOpponents — 2 cards

- **TODOs**: 3
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: drana_and_linvala, elesh_norn_mother_of_machines
- **Sample TODO**: "Static — activated abilities of opponents' creatures can't be activated."

### 105. LayerModification::TypeChange — 2 cards

- **TODOs**: 3
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: eaten_by_piranhas, enduring_curiosity
- **Sample TODO**: "Color override (becomes black only) — needs LayerModification::SetColors."

### 106. Effect::ReturnToHand — 2 cards

- **TODOs**: 2
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: encroaching_dragonstorm, roiling_dragonstorm
- **Sample TODO**: "(second trigger): Effect::ReturnToHand does not exist as a DSL variant."

### 107. Effect::ExileGraveyard — 2 cards

- **TODOs**: 2
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: farewell, rakdos_charm
- **Sample TODO**: ""exile all graveyards" — ExileAll only targets battlefield"

### 108. Cost::DiscardCreatureCard — 2 cards

- **TODOs**: 2
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: fauna_shaman, tortured_existence
- **Sample TODO**: "{G}, {T}, Discard a creature card: search for creature card, put into hand"

### 109. TriggerCondition::WhenExploits — 2 cards

- **TODOs**: 4
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: fell_stinger, sidisi_undead_vizier
- **Sample TODO**: "exploit trigger draw/lose effect with target player requires targeted_trigger"

### 110. TriggerCondition::WheneverPlayerCastsSpell — 2 cards

- **TODOs**: 2
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: forgotten_ancient, niv_mizzet_parun
- **Sample TODO**: "DSL gap — "Whenever a player casts a spell" trigger condition."

### 111. EffectAmount::CardTypeCount — 2 cards

- **TODOs**: 3
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: keen_eyed_curator, nethergoyf
- **Sample TODO**: "DSL gap — conditional static buff based on counting distinct card types among"

### 112. TriggerCondition::WheneverYouSacrifice — 2 cards

- **TODOs**: 2
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: korvold_fae_cursed_king, smothering_abomination
- **Sample TODO**: ""Whenever you sacrifice a permanent" trigger not in DSL."

### 113. Cost::SacrificeAnyNumber — 2 cards

- **TODOs**: 3
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: last_ditch_effort, scapeshift
- **Sample TODO**: ""Sacrifice any number of creatures" — no Cost::SacrificeAnyNumber or"

### 114. Keyword::Lieutenant — 2 cards

- **TODOs**: 2
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: loyal_apprentice, siege_gang_lieutenant
- **Sample TODO**: "Lieutenant ability — requires intervening-if Condition::YouControlYourCommander"

### 115. TargetFilter::NonbasicLand — 2 cards

- **TODOs**: 3
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: magmatic_hellkite, wasteland
- **Sample TODO**: "DSL gap — ETB trigger targeting a nonbasic land (non_land filter won't work — need"

### 116. Static::AllCreatureTypes — 2 cards

- **TODOs**: 2
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: maskwood_nexus, mirror_entity
- **Sample TODO**: "Static — "Creatures you control are every creature type" — DSL lacks a"

### 117. Keyword::Soulbond — 2 cards

- **TODOs**: 2
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: mirage_phalanx, tandem_lookout
- **Sample TODO**: ""loses soulbond" on copy token not expressible."

### 118. LayerModification::SetBothDynamic — 2 cards

- **TODOs**: 2
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: mirror_entity, skyclave_apparition
- **Sample TODO**: "Dynamic P/T setting (base X/X) requires LayerModification::SetBothDynamic(EffectAmount)"

### 119. Effect::ForEach — 2 cards

- **TODOs**: 3
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: nanogene_conversion, syr_konrad_the_grim
- **Sample TODO**: ""each other creature becomes a copy" — needs ForEach over all creatures +"

### 120. TriggerCondition::WhenOpponentDraws — 2 cards

- **TODOs**: 2
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: orcish_bowmasters, razorkin_needlehead
- **Sample TODO**: ""whenever an opponent draws a card except the first one they draw in"

### 121. Effect::SearchMultipleCards — 2 cards

- **TODOs**: 2
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: protean_hulk, sarkhan_unbroken
- **Sample TODO**: "DSL gap — death trigger + multi-card search with total MV constraint."

### 122. Static::EquipmentGrants — 2 cards

- **TODOs**: 2
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: quietus_spike, sting_the_glinting_dagger
- **Sample TODO**: "DSL gap — "equipped creature has deathtouch" requires equipment continuous effect;"

### 123. Effect::Proliferate — 2 cards

- **TODOs**: 2
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: roalesk_apex_hybrid, voidwing_hybrid
- **Sample TODO**: "DSL gap — "When Roalesk dies, proliferate, then proliferate again.""

### 124. Static::GrantKeywordsConditional — 2 cards

- **TODOs**: 3
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: scion_of_draco, sting_the_glinting_dagger
- **Sample TODO**: "DSL gap — static ability: "Each creature you control has vigilance if it's white,"

### 125. Keyword::Warp — 2 cards

- **TODOs**: 3
- **Estimated LOC**: ~70
- **Existing PB**: new
- **Cards**: starfield_shepherd, timeline_culler
- **Sample TODO**: "Warp keyword is not in the DSL (KeywordAbility enum). No AltCostKind::Warp exists."

### 126. Keyword::Compleated — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: ajani_sleeper_agent
- **Sample TODO**: "Compleated keyword — 2 fewer loyalty if Phyrexian life was paid."

### 127. Static::CantBeSacrificed — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: alexios_deimos_of_kosmos
- **Sample TODO**: ""can't be sacrificed" — sacrifice restriction not in DSL"

### 128. EnchantTarget::GraveyardCard — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: animate_dead
- **Sample TODO**: "Complex reanimation Aura — enchants graveyard card (no EnchantTarget variant for"

### 129. LayerModification::ModifyBothWithEffectAmount — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: PB-X
- **Cards**: aspect_of_hydra
- **Sample TODO**: "DSL gap — LayerModification::ModifyBoth(i32) takes a static i32, not EffectAmount."

### 130. Effect::AddCounters — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: avenger_of_zendikar
- **Sample TODO**: ""Each Plant you control" counter distribution not in DSL."

### 131. Effect::AttachAllEquipment — 1 cards

- **TODOs**: 3
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: balan_wandering_knight
- **Sample TODO**: "DSL gap — conditional double strike requiring count of attached Equipment (2+)"

### 132. TriggerCondition::WhenCounterSpell — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: baral_chief_of_compliance
- **Sample TODO**: ""Whenever you counter a spell" trigger not in DSL."

### 133. Static::ForcedAttack — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: bident_of_thassa
- **Sample TODO**: ""{1}{U}, {T}: Creatures your opponents control attack this turn if able.""

### 134. Effect::SearchWithConstraint — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: birthing_pod
- **Sample TODO**: "{1}{G/P}, {T}, Sacrifice a creature: search for creature with MV = sacrificed MV + 1"

### 135. Static::AdditionalBlocker — 1 cards

- **TODOs**: 2
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: brave_the_sands
- **Sample TODO**: "DSL gap — "Each creature you control can block an additional creature"

### 136. TriggerCondition::WhenBecomesBlocked — 1 cards

- **TODOs**: 2
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: cunning_evasion
- **Sample TODO**: "TriggerCondition::WhenBecomesBlocked does not exist in DSL."

### 137. EnchantTarget::EnchantPlayer — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: curse_of_opulence
- **Sample TODO**: ""Enchant player" not in EnchantTarget enum."

### 138. TriggerCondition::WheneverEnchantedCreatureAttacks — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: curse_of_opulence
- **Sample TODO**: ""Whenever enchanted player is attacked" trigger not in DSL."

### 139. Keyword::SpellMastery — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: dark_petition
- **Sample TODO**: "Spell mastery conditional mana bonus ({B}{B}{B} if 2+ instant/sorcery in"

### 140. Effect::LoseLifeEqualTo — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: deadly_tempest
- **Sample TODO**: "The "each player loses life equal to creatures they controlled" requires"

### 141. LayerModification::DoubleCreaturePower — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: devilish_valet
- **Sample TODO**: "DSL gap — "double this creature's power" continuous effect not expressible"

### 142. Keyword::Alliance — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: devilish_valet
- **Sample TODO**: "Alliance trigger — "double this creature's power until EOT""

### 143. Keyword::Transmute — 1 cards

- **TODOs**: 2
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: dimir_infiltrator
- **Sample TODO**: "KeywordAbility::Transmute does not exist in the DSL. The transmute activated"

### 144. ComplexPattern::NameTracking — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: dragonlord_kolaghan
- **Sample TODO**: "DSL gap — the triggered ability requires checking the opponent's graveyard for a name match,"

### 145. Keyword::Flashback — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: dread_return
- **Sample TODO**: "Flashback cost is "Sacrifice three creatures" — not a mana cost. The"

### 146. Keyword::Rebound — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: ephemerate
- **Sample TODO**: "Rebound keyword — not yet in the DSL KeywordAbility enum. Implementing the flicker"

### 147. TriggerCondition::WhenProliferate — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: ezuri_stalker_of_spheres
- **Sample TODO**: ""Whenever you proliferate" trigger not in DSL."

### 148. EffectAmount::ColorCount — 1 cards

- **TODOs**: 2
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: faeburrow_elder
- **Sample TODO**: "DSL gap — "gets +1/+1 for each color among permanents you control" is a static"

### 149. TriggerCondition::WhenCommitCrime — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: forsaken_miner
- **Sample TODO**: ""Whenever you commit a crime" — TriggerCondition::WheneverYouCommitACrime"

### 150. Effect::DiscardAtRandom — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: gamble
- **Sample TODO**: "Effect::DiscardAtRandom does not exist in the DSL. The "discard a card at random""

### 151. Effect::ExchangeControl — 1 cards

- **TODOs**: 2
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: gilded_drake
- **Sample TODO**: "ETB exchange-control targeted trigger not in DSL"

### 152. Keyword::ForMirrodin — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: glimmer_lens
- **Sample TODO**: ""For Mirrodin!" — ETB token + auto-attach not expressible."

### 153. TriggerCondition::WheneverEquippedCreatureAttacks — 1 cards

- **TODOs**: 2
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: glimmer_lens
- **Sample TODO**: ""Equipped creature + another attack" trigger not expressible."

### 154. Static::TreasuresManaBonus — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: goldspan_dragon
- **Sample TODO**: ""Treasures add two mana" static override not in DSL."

### 155. Replacement::ShuffleInsteadOfGraveyard — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: green_suns_zenith
- **Sample TODO**: ""shuffle into library instead of graveyard" replacement."

### 156. Keyword::Demonstrate — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: incarnation_technique
- **Sample TODO**: "Demonstrate is a keyword that triggers when the spell is cast and lets"

### 157. Replacement::DamagePreventionVariable — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: inkshield
- **Sample TODO**: "Prevention effect + variable token count based on damage prevented not in DSL"

### 158. Interactive::MayPayOrElse — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: izzet_charm
- **Sample TODO**: ""unless its controller pays {2}" — conditional counter not in DSL."

### 159. ComplexPattern::EndStepTrigger — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: knight_of_the_ebon_legion
- **Sample TODO**: "DSL gap — end step trigger with "if a player lost 4+ life this turn""

### 160. Keyword::Mentor — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: legion_warboss
- **Sample TODO**: "Mentor keyword not in DSL."

### 161. LayerModification::ColorChange — 1 cards

- **TODOs**: 2
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: leyline_of_the_guildpact
- **Sample TODO**: ""Each nonland permanent you control is all colors" — layer 5 color modification for"

### 162. Replacement::ETBAsOtherType — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: master_biomancer
- **Sample TODO**: ""enters as a Mutant in addition to its other types" — type-granting ETB replacement"

### 163. TriggerCondition::WhenUntaps — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: mesmeric_orb
- **Sample TODO**: "TriggerCondition::WheneverPermanentUntaps not in DSL."

### 164. TargetFilter::SupertypeConstraint — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: minamo_school_at_waters_edge
- **Sample TODO**: "Target should be "legendary permanent" — TargetFilter lacks supertype constraint."

### 165. ComplexPattern::CopyWithModification — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: mirage_phalanx
- **Sample TODO**: ""Create a token that's a copy of this creature, except it has haste and loses"

### 166. Keyword::Meld — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: mishra_claimed_by_gix
- **Sample TODO**: "Meld trigger (Phyrexian Dragon Engine + Mishra meld) — Meld not yet in DSL."

### 167. TriggerCondition::WhenCommanderEntersOrAttacks — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: norns_choirmaster
- **Sample TODO**: ""Whenever a commander you control enters or attacks, proliferate" —"

### 168. TriggerCondition::WheneverCreatureEnters — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: ogre_battledriver
- **Sample TODO**: "WheneverCreatureEntersBattlefield trigger — ETBTriggerFilter available,"

### 169. Replacement::CounterReplacement — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: oran_rief_hydra
- **Sample TODO**: "DSL gap — "If that land is a Forest, put two counters instead.""

### 170. TriggerCondition::EquipmentEnters — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: puresteel_paladin
- **Sample TODO**: ""Equipment enters" trigger — WheneverPermanentEntersBattlefield with"

### 171. Keyword::Metalcraft — 1 cards

- **TODOs**: 2
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: puresteel_paladin
- **Sample TODO**: "Metalcraft conditional equip cost reduction not expressible."

### 172. Keyword::Renew — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: qarsi_revenant
- **Sample TODO**: "Renew activated ability cannot be expressed — it activates from the graveyard"

### 173. Keyword::Miracle — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: reforge_the_soul
- **Sample TODO**: "Miracle {1}{R} — KeywordAbility::Miracle not yet implemented."

### 174. Static::PlayerHexproof — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: shalai_voice_of_plenty
- **Sample TODO**: "DSL gap — "You have hexproof" (player hexproof) not in layer system."

### 175. Keyword::Ferocious — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: shamanic_revelation
- **Sample TODO**: "Ferocious — gain 4 life for each creature with power 4+."

### 176. Effect::Goad — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: shiny_impetus
- **Sample TODO**: ""Enchanted creature is goaded" — static goad applied to attached creature"

### 177. Keyword::Saga — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: song_of_freyalise
- **Sample TODO**: "Saga chapter I/II — grant via PB-S LayerModification::AddManaAbility"

### 178. Keyword::Hideaway — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: spinerock_knoll
- **Sample TODO**: "Keyword — Hideaway 4"

### 179. Static::DrawLimitPerTurn — 1 cards

- **TODOs**: 2
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: spirit_of_the_labyrinth
- **Sample TODO**: ""Each player can't draw more than one card each turn" — a static restriction on"

### 180. TargetFilter::ManaValueLessOrEqual — 1 cards

- **TODOs**: 2
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: starfield_shepherd
- **Sample TODO**: "ETB search filter requires OR semantics: "basic Plains" OR "creature MV ≤ 1"."

### 181. Keyword::Madness — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: stensia_masquerade
- **Sample TODO**: "Madness {2}{R} — AltCostKind::Madness not in DSL."

### 182. Keyword::Cycling — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: tectonic_reformation
- **Sample TODO**: "Grant cycling to hand lands not expressible."

### 183. Effect::WinGame — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: thassas_oracle
- **Sample TODO**: ""If X >= library size, you win the game" — no Effect::WinGame or condition that checks"

### 184. Keyword::Morbid — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: tragic_slip
- **Sample TODO**: "Morbid — "if a creature died this turn" → -13/-13, else -1/-1."

### 185. Keyword::Addendum — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: unbreakable_formation
- **Sample TODO**: "DSL gap — Addendum: "if during main phase" condition check"

### 186. Keyword::Escape — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: uro_titan_of_natures_wrath
- **Sample TODO**: ""Sacrifice unless it escaped" ETB — needs cast_alt_cost check."

### 187. Static::PlayWithTopRevealed — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: vampire_nocturnus
- **Sample TODO**: "DSL gap — "Play with the top card of your library revealed.""

### 188. Static::HexproofFromAbilities — 1 cards

- **TODOs**: 1
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: volatile_stormdrake
- **Sample TODO**: ""Hexproof from activated and triggered abilities" is a partial-source"

### 189. TriggerCondition::WheneverYouGainLife — 1 cards

- **TODOs**: 2
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: well_of_lost_dreams
- **Sample TODO**: ""Whenever you gain life, pay {X} up to the amount gained, draw X cards" —"

### 190. Effect::WheelEffect — 1 cards

- **TODOs**: 2
- **Estimated LOC**: ~60
- **Existing PB**: new
- **Cards**: winds_of_change
- **Sample TODO**: ""shuffle hand into library, draw that many" — wheel effect. No Effect to"

---

## Uncategorized TODOs

**Count**: 204 TODOs from 176 cards

These need manual classification or are unique edge cases:

- `abzan_charm`: distributing counters across two separately-declared targets is not
- `ainok_strike_leader`: "Whenever you attack with this creature and/or your commander, for each
- `ajani_sleeper_agent`: distributed counter placement + grant vigilance until EOT.
- `ajani_sleeper_agent`: Spell-type filter (creature/planeswalker only) is not enforced —
- `allosaurus_shepherd`: Activated overwrite: base P/T 5/5 + add Dinosaur type to Elves.
- `bloodghast`: Oracle says "you may return" — currently non-optional (bot always returns).
- `bonecrusher_giant`: (2): Effect should target "that spell's controller" — needs EffectTarget::Trigge...
- `bonecrusher_giant`: (2): Effect target is WRONG — should deal 2 damage to "that spell's controller"
- `bonecrusher_giant`: (3): "Damage can't be prevented this turn" — no Effect::PreventionShieldRemoval
- `bonecrusher_giant`: "Damage can't be prevented this turn" — no prevention-removal DSL effect.
- `braids_arisen_nightmare`: Complex end-step trigger with sacrifice choice, type-matching opponent sacrifice...
- `canopy_crawler`: activated ability — {T}: target creature gets +1/+1 until end of turn
- `carth_the_lion`: ETB trigger — "look at the top seven cards, may put a planeswalker card
- `carth_the_lion`: "Planeswalkers' loyalty abilities you activate cost an additional [+1]"
- `chrome_mox`: ETB optional exile from hand (Imprint) and "add mana of exiled card's colors"
- `cover_of_darkness`: "As this enters, choose a creature type" — no ChooseCreatureType
- `crucible_of_the_spirit_dragon`: {1}, {T}: Put a storage counter on this land.
- `crystal_barricade`: "Prevent all noncombat damage that would be dealt to other creatures you
- `culling_ritual`: When variable-count mana generation with color choice is supported,
- `curiosity`: (PB-37): approximation — oracle says "an opponent" but
- `darksteel_garrison`: TriggerCondition::WhenFortifiedLandBecomesTapped does not exist yet.
- `dawns_truce`: Gift mechanic draw + hexproof/indestructible continuous effects complex for DSL.
- `deep_gnome_terramancer`: "lands enter under opponent's control without being played" trigger condition
- `deep_gnome_terramancer`: Mold Earth triggered ability — needs TriggerCondition::WheneverOpponentGetsLandW...
- `delney_streetwise_lookout`: Trigger doubling for low-power creatures — needs a power-filtered
- `den_protector`: Static evasion — "Creatures with power less than this creature's power
- `destiny_spinner`: static "can't counter creature/enchantment spells you control"
- `destiny_spinner`: static "can't counter creature/enchantment spells you control"
- `dockside_extortionist`: "X = artifacts + enchantments opponents control" — count-based
- `dolmen_gate`: Effect::PreventAllCombatDamage prevents ALL combat damage (attacker and defender...
- ... and 174 more

---

## Recommended PB Priority (based on yield analysis)

Sorted by **cards unblocked × yield confidence**:

| Rank | Primitive | Cards | Yield Est | Priority |
|------|-----------|-------|-----------|----------|
| 1 | TriggerCondition::SubtypeFilteredAttack | 18 | ~60% = 11 | **HIGH** — one dispatch site |
| 2 | TargetFilter::DamagedPlayer | 15 | ~60% = 9 | **HIGH** — ForEach + targeting |
| 3 | TriggerCondition::SubtypeFilteredDeath | 15 | ~60% = 9 | HIGH — sibling to attack |
| 4 | TriggerCondition::Landfall | 13 | ~50% = 7 | HIGH — common pattern |
| 5 | EffectAmount::PowerOfCreature | 13 | ~70% = 9 | HIGH — EffectAmount variant |
| 6 | TriggerCondition::WheneverOpponentLosesLife | 11 | ~50% = 6 | MED — opponent tracking |
| 7 | Cost::TapNCreatures | 10 | ~60% = 6 | MED — cost framework |
| 8 | Cost::SacrificeAnother | 10 | ~60% = 6 | MED — cost framework |
| 9 | Effect::NoMaxHandSize | 5 | ~80% = 4 | LOW — simple static |
| 10 | TargetFilter::UpToN | 21 | ~30% = 6 | LOW — PB-T (already planned) |

**Notes**:
- Interactive::PlayerChoice (21 cards) → M10+ deferred, 0% yield now
- ComplexPattern::ConditionalIf (37 cards) → too heterogeneous for one PB
- DSL Gap (unclassified) (135 cards) → requires per-card triage

---

## Next Steps

1. **PB-Q4** (in progress): EnchantTarget::Filtered — 5 cards expected
2. **After PB-Q4**: Pick from top 5 above based on current workstream
3. **Stale TODO audit**: Check song_of_freyalise, bootleggers_stash for PB-S unblock
4. **M10 prep**: The 21 Interactive::PlayerChoice cards become unlocked after M10