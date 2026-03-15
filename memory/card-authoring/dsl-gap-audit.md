# DSL Gap Audit — Card Definition Files

> **Date**: 2026-03-13
> **Scope**: All 718 card def files in `crates/engine/src/cards/defs/`
> **Method**: grep-based TODO extraction + pattern categorization + partial-implementation scan
> **Status**: Research only — no files modified

---

## Executive Summary

| Metric | Count |
|--------|-------|
| Total card def files | 718 |
| Files with TODO comments | 418 (58%) |
| Files with `abilities: vec![]` | 80 (11%) |
| Files with empty abilities AND no TODO | 43 (6%) |
| Total TODO lines | 793 |
| Dangerous partial implementations | 122 |
| Distinct gap buckets | 47 |

**Top 5 gaps by card count**: activated_ability_complex (114), triggered_ability_complex (86), conditional_etb_tapped (56), targeted_activated_ability (32), static_grant_conditional (30).

**Most dangerous gaps**: 78 cards have ETB-tapped noted in TODO but `etb_tapped: true` not set (lands enter untapped when they should enter tapped). 8 pain lands produce mana without dealing damage. 5 cards have approximated mana costs (hybrid -> mono).

---

## Summary Table (sorted by card count)

| # | Gap Bucket | Cards | Complexity | Notes |
|---|-----------|------:|------------|-------|
| 1 | `activated_ability_complex` | 114 | Med-High | Catch-all for {T}:, {N},{T}: abilities |
| 2 | `triggered_ability_complex` | 86 | Med-High | Catch-all for "whenever", "when", "at the beginning" |
| 3 | `conditional_etb_tapped` | 56 | Low-Med | "enters tapped unless..." conditions |
| 4 | `targeted_activated_ability` | 32 | Med | "Target creature/permanent" in activated/triggered |
| 5 | `static_grant_conditional` | 30 | Med | Layer 6 "creatures you control have/get..." |
| 6 | `count_based_scaling` | 29 | Med | "for each", "number of", dynamic counts |
| 7 | `etb_tapped_missing` | 28 | Low | Simple enters-tapped not set (includes 10 shocklands) |
| 8 | `sacrifice_cost` | 26 | Med | "Sacrifice a/this" as activation cost |
| 9 | `replacement_effect` | 11 | High | Modifying events as they happen |
| 10 | `cost_reduction` | 10 | Med-High | "spells cost {1} less/more" statics |
| 11 | `mana_with_damage` | 8 | Low | Pain lands: mana + self-damage |
| 12 | `return_from_zone` | 8 | Med | Return card from GY to hand/BF/library |
| 13 | `etb_player_choice` | 7 | Med | "as this enters, choose a creature type" |
| 14 | `cda_power_toughness` | 6 | Med | Characteristic-defining P/T |
| 15 | `modal_complex` | 6 | Med | Per-mode targets, conditional modal |
| 16 | `mana_spending_restriction` | 6 | Med-High | "Spend this mana only to cast..." |
| 17 | `look_at_top` | 6 | Med | Reveal/look at top N, conditionally put |
| 18 | `hybrid_mana_cost` | 5 | Med | {W/B}, {G/W/P} not in ManaCost |
| 19 | `combat_restriction` | 5 | Low-Med | "can't block", "can't attack owner" |
| 20 | `cycling` | 5 | **None** | DSL supports it (`Cycling { cost }`); just not wired |
| 21 | `planeswalker` | 4 | **Very High** | CardType + loyalty abilities missing entirely |
| 22 | `graveyard_play` | 4 | Med | "play/cast from graveyard" static |
| 23 | `keyword_grant_continuous` | 4 | Med | Conditional haste, subtype-filtered grants |
| 24 | `counter_distribution` | 3 | Med | Distributing/removing specific counters |
| 25 | `forced_attack` | 3 | Low | "attacks each combat if able" |
| 26 | `land_animation` | 3 | Med-High | "{1}: becomes a creature until EOT" |
| 27 | `filter_creatures_you_control` | 3 | Low-Med | `EffectFilter::CreaturesYouControl` missing |
| 28 | `library_search` | 3 | Med | Search GY, search with complex filters |
| 29 | `damage_prevention` | 3 | Med | "Prevent all damage to attacking creatures" |
| 30 | `channel_ability` | 3 | Med | Discard-from-hand alt cost |
| 31 | `token_creation_complex` | 2 | Med-High | Token doubling, dynamic P/T tokens |
| 32 | `coin_flip` | 2 | Med | d20 rolls, coin flips |
| 33 | `ascend` | 2 | Med | City's Blessing designation |
| 34 | `equipment_attach` | 2 | Med | Auto-attach + equipped-creature grants |
| 35 | `x_cost` | 2 | Med | ManaCost lacks X field |
| 36 | `timing_restriction` | 2 | Med-High | "opponents can't cast during your turn" |
| 37 | `etb_control` | 2 | Med | ETB gain control of permanents |
| 38 | `copy_effect` | 2 | High | Ability copying, ETB clone |
| 39 | `saga_class_mechanic` | 2 | High | Saga lore counters, Class levels |
| 40 | `wither` | 1 | Low | KW variant needed (CR 702.77) |
| 41 | `player_hexproof` | 1 | Low | "you have hexproof" |
| 42 | `evasion_static` | 1 | Med | Power-based blocking restriction |
| 43 | `color_indicator` | 1 | Low | Dryad Arbor green indicator |
| 44 | `dredge` | 1 | Med | AbilityDefinition::Dredge needed |
| 45 | `meld` | 1 | High | Meld mechanic |
| 46 | `buyback` | 1 | Med | KW variant needed |
| 47 | `exile_warp` | 1 | Med | Warp AltCostKind |

---

## Detailed Gap Listings

### 1. `activated_ability_complex` (114 cards) — Med-High

The largest bucket. These cards have activated abilities (typically `{N}, {T}: <effect>`) that are either entirely missing or partially implemented. Many overlap with other gaps (targeting, sacrifice costs, mana restrictions).

**Cards**: abstergo_entertainment, alseid_of_lifes_bounty, arch_of_orazca, arena_of_glory, arixmethes_slumbering_isle, balan_wandering_knight, bladewing_the_risen, blazemire_verge, bleachbone_verge, blinkmoth_nexus, blood_crypt, boseiju_who_endures, brash_taunter, breeding_pool, cascade_bluffs, castle_ardenvale, castle_embereth, castle_locthwain, castle_vantress, command_beacon, creeping_tar_pit, crown_of_skemfar, crucible_of_the_spirit_dragon, crypt_of_agadeem, cryptic_coat, cult_conscript, den_of_the_bugbear, deserted_temple, drana_and_linvala, drivnod_carnage_dominus, eiganjo_seat_of_the_empire, fetid_heath, flamekin_village, flooded_grove, forbidden_orchard, forerunner_of_slaughter, geier_reach_sanitarium, gemstone_caverns, gingerbrute, glistening_sphere, gloomlake_verge, gnarlroot_trapper, godless_shrine, graven_cairns, gruul_turf, halimar_depths, hall_of_heliods_generosity, hallowed_fountain, hanweir_battlements, haven_of_the_spirit_dragon, indatha_triome, karns_bastion, kher_keep, kor_haven, maelstrom_of_the_spirit_dragon, mina_and_denn_wildborn, minamo_school_at_waters_edge, minas_tirith, mistrise_village, mortuary_mire, mothdust_changeling, mother_of_runes, multani_yavimayas_avatar, mystic_sanctuary, necrogen_rotpriest, oboro_palace_in_the_clouds, oran_rief_the_vastwood, otawara_soaring_city, overgrown_tomb, phyrexian_tower, qarsi_revenant, quietus_spike, raugrin_triome, reassembling_skeleton, rugged_prairie, sacred_foundry, savai_triome, scion_of_the_ur_dragon, secluded_courtyard, shadowspear, shifting_woodland, shizo_deaths_storehouse, skemfar_elderhall, skithiryx_the_blight_dragon, skrelv_defector_mite, sparas_headquarters, spinerock_knoll, steam_vents, stomping_ground, sunken_palace, sunken_ruins, tainted_field, tainted_isle, tainted_wood, takenuma_abandoned_mire, tekuthal_inquiry_dominus, temple_garden, temple_of_the_dragon_queen, temple_of_the_false_god, temur_sabertooth, the_seedcore, thespians_stage, three_tree_city, twilight_mire, valakut_the_molten_pinnacle, vault_of_the_archangel, volatile_stormdrake, voldaren_estate, war_room, wastewood_verge, watery_grave, wirewood_lodge, witchs_cottage, ziatoras_proving_ground

### 2. `triggered_ability_complex` (86 cards) — Med-High

Cards with triggered abilities ("whenever", "when", "at the beginning") that are not implemented. Many involve complex trigger conditions not yet in the DSL.

**Cards**: alela_cunning_conqueror, alexios_deimos_of_kosmos, ancient_brass_dragon, ancient_greenwarden, arixmethes_slumbering_isle, aven_riftwatcher, azorius_chancery, balefire_dragon, biting_palm_ninja, bloodthirsty_conqueror, boromir_warden_of_the_tower, boros_garrison, camellia_the_seedmiser, darksteel_garrison, dimir_aqueduct, dragonlord_ojutai, dreadhorde_invasion, duelists_heritage, emeria_the_sky_ruin, emrakul_the_promised_end, endurance, fell_stinger, field_of_the_dead, florian_voldaren_scion, flowerfoot_swordmaster, frantic_scapegoat, glistening_sphere, golgari_rot_farm, grief, gruul_turf, halimar_depths, hammer_of_nazahn, hammerhead_tyrant, hellkite_tyrant, inventors_fair, ixhel_scion_of_atraxa, izzet_boilerworks, karlach_fury_of_avernus, kodama_of_the_east_tree, legion_loyalist, lozhan_dragons_legacy, magmatic_hellkite, mana_crypt, mindleecher, mishra_claimed_by_gix, moria_marauder, mossborn_hydra, mystic_remora, mystic_sanctuary, necrogen_rotpriest, niv_mizzet_visionary, nullpriest_of_oblivion, ogre_battledriver, orzhov_basilica, phyrexian_dreadnought, pitiless_plunderer, quilled_charger, rakdos_carnarium, razorkin_needlehead, seasoned_dungeoneer, selesnya_sanctuary, shambling_ghast, simic_growth_chamber, skullclamp, sky_hussar, slickshot_show_off, smoke_shroud, spinerock_knoll, sting_the_glinting_dagger, sylvan_messenger, tainted_observer, tatyova_steward_of_tides, teneb_the_harvester, terror_of_the_peaks, teysa_karlov, thieving_skydiver, thousand_faced_shadow, tombstone_stairwell, toothy_imaginary_friend, twilight_prophet, twinflame_tyrant, valakut_the_molten_pinnacle, volatile_stormdrake, warren_instigator, wrathful_red_dragon, zurgo_and_ojutai

### 3. `conditional_etb_tapped` (56 cards) — Low-Med

Lands that enter tapped unless a condition is met. The DSL has `etb_tapped: true` but lacks conditional ETB-tapped (replacement effects checking board state).

**Sub-patterns**:
- "unless you control a [land type]" — check-lands (12 cards): clifftop_retreat, dragonskull_summit, drowned_catacomb, glacial_fortress, hinterland_harbor, isolated_chapel, rootbound_crag, sulfur_falls, sunpetal_grove, woodland_cemetery, castle_ardenvale/embereth/locthwain/vantress
- "unless you control two or fewer other lands" — fast-lands (6): blooming_marsh, concealed_courtyard, darkslick_shores, den_of_the_bugbear, shifting_woodland + variants
- "unless you have two or more opponents" — bond-lands (6): bountiful_promenade, luxury_suite, morphic_pool, sea_of_clouds, spectator_seating, spire_garden + variants
- "reveal a [type] card from your hand" — reveal-lands (6): choked_estuary, foreboding_ruins, frostboil_snarl, furycalm_snarl, gilt_leaf_palace, shineshadow_snarl + variants
- "unless you control two or more basic lands" — battle-lands (4): canopy_vista, prairie_stream, smoldering_marsh, sunken_hollow + cinder_glade

**Cards**: arena_of_glory, blooming_marsh, bountiful_promenade, canopy_vista, castle_ardenvale, castle_embereth, castle_locthwain, castle_vantress, choked_estuary, cinder_glade, clifftop_retreat, concealed_courtyard, darkslick_shores, deathcap_glade, dragonskull_summit, dreamroot_cascade, drowned_catacomb, flamekin_village, foreboding_ruins, frostboil_snarl, furycalm_snarl, gilt_leaf_palace, glacial_fortress, haunted_ridge, hinterland_harbor, isolated_chapel, luxury_suite, minas_tirith, mistrise_village, morphic_pool, mystic_sanctuary, necroblossom_snarl, prairie_stream, rejuvenating_springs, rockfall_vale, rootbound_crag, sea_of_clouds, shattered_sanctum, shifting_woodland, shineshadow_snarl, shipwreck_marsh, smoldering_marsh, spectator_seating, spire_garden, spymasters_vault, stormcarved_coast, sulfur_falls, sundown_pass, sunken_hollow, sunpetal_grove, temple_of_the_dragon_queen, training_center, undergrowth_stadium, vault_of_champions, witchs_cottage, woodland_cemetery

### 4. `targeted_activated_ability` (32 cards) — Med

Activated or triggered abilities that require targeting a specific permanent/player, where `AbilityDefinition::Activated` lacks a `targets` field.

**Cards**: access_tunnel, blinkmoth_nexus, bloodline_necromancer, bojuka_bog, brash_taunter, briarblade_adept, canopy_crawler, elder_deep_fiend, endurance, eomer_king_of_rohan, fell_stinger, flamekin_village, forbidden_orchard, gilded_drake, goblin_motivator, great_oak_guardian, grief, hammerhead_tyrant, hanweir_battlements, mortuary_mire, necrogen_rotpriest, nullpriest_of_oblivion, quietus_spike, reanimate, sharktocrab, shifting_woodland, slayers_stronghold, spymasters_vault, teneb_the_harvester, witchs_cottage, yavimaya_hollow, zealous_conscripts

### 5. `static_grant_conditional` (30 cards) — Med

Layer 6 continuous effects granting keywords or P/T bonuses to subsets of creatures. Requires `EffectFilter` variants that don't exist yet.

**Cards**: ainok_bond_kin, akromas_will, archetype_of_endurance, blade_historian, bloodmark_mentor, brave_the_sands, camellia_the_seedmiser, castle_embereth, crashing_drawbridge, cryptic_coat, dragonlord_kolaghan, etchings_of_the_chosen, final_showdown, goblin_war_drums, great_oak_guardian, greymond_avacyns_stalwart, iroas_god_of_victory, karrthus_tyrant_of_jund, legolasquick_reflexes, legolass_quick_reflexes, markov_baron, mass_hysteria, overwhelming_stampede, reckless_bushwhacker, rhythm_of_the_wild, tatyova_steward_of_tides, teysa_karlov, thousand_year_elixir, throatseeker, ultramarines_honour_guard

### 6. `count_based_scaling` (29 cards) — Med

Effects that scale based on counting objects (creatures, lands, card types, etc.).

**Cards**: balan_wandering_knight, blasphemous_act, cabal_coffers, cabal_stronghold, call_of_the_ring, craterhoof_behemoth, crimestopper_sprite, crown_of_skemfar, crypt_of_agadeem, destiny_spinner, devilish_valet, emrakul_the_promised_end, eomer_king_of_rohan, faeburrow_elder, frodo_saurons_bane, gaeas_cradle, ghalta_primal_hunger, indomitable_archangel, jagged_scar_archers, malakir_bloodwitch, multani_yavimayas_avatar, nykthos_shrine_to_nyx, reckless_one, scion_of_draco, scryb_ranger, the_ur_dragon, three_tree_city, tombstone_stairwell, war_room

### 7. `etb_tapped_missing` (28 cards) — Low

Simple enters-tapped that could be fixed immediately by setting `etb_tapped: true`. Includes 10 shocklands (which need the "pay 2 life" replacement effect too).

**Shocklands** (10, need pay-2-life-or-tapped replacement): blood_crypt, breeding_pool, godless_shrine, hallowed_fountain, overgrown_tomb, sacred_foundry, steam_vents, stomping_ground, temple_garden, watery_grave

**Simple enters-tapped** (18): creeping_tar_pit, crypt_of_agadeem, den_of_the_bugbear, glistening_sphere (artifact), gruul_turf, halimar_depths, indatha_triome, mortuary_mire, oathsworn_vampire (creature), oran_rief_the_vastwood, raugrin_triome, savai_triome, skemfar_elderhall, sparas_headquarters, spinerock_knoll, sunken_palace, valakut_the_molten_pinnacle, ziatoras_proving_ground

### 8. `sacrifice_cost` (26 cards) — Med

Abilities that require sacrificing a permanent (self or other) as part of the cost.

**Cards**: alexios_deimos_of_kosmos, boromir_warden_of_the_tower, buried_ruin, camellia_the_seedmiser, command_beacon, crop_rotation, deadly_dispute, etchings_of_the_chosen, flare_of_fortitude, ghost_quarter, gingerbrute, grim_backwoods, haven_of_the_spirit_dragon, high_market, hope_of_ghirapur, inventors_fair, maelstrom_of_the_spirit_dragon, phyrexian_tower, scavenger_grounds, skemfar_elderhall, strip_mine, the_world_tree, torch_courier, treasure_vault, vampire_hexmage, wasteland

### 9. `replacement_effect` (11 cards) — High

Complex replacement effects that modify events as they happen.

**Cards**: adrix_and_nev_twincasters, archon_of_emeria, aven_mindcensor, bloodletter_of_aclazotz, crimestopper_sprite, gemrazer, neriv_heart_of_the_storm, pir_imaginative_rascal, tekuthal_inquiry_dominus, twinflame_tyrant, vorinclex_monstrous_raider

### 10. `cost_reduction` (10 cards) — Med-High

Static effects that reduce or increase spell costs.

**Cards**: danitha_capashen_paragon, emrakul_the_promised_end, ghalta_primal_hunger, goblin_warchief, jhoiras_familiar, otawara_soaring_city, sokenzan_crucible_of_defiance, thalia_guardian_of_thraben, the_ur_dragon, voldaren_estate

### 11-47. Remaining Gaps

See summary table above for counts. Notable entries:

- **`mana_with_damage`** (8): All pain lands. Mana is produced but damage is omitted. Dangerous for testing.
- **`cycling`** (5): DSL supports `Cycling { cost }` already. These just need to be wired up. Zero implementation effort.
- **`planeswalker`** (4): ajani_sleeper_agent, strix_serenade (targets planeswalkers), tyvar_jubilant_brawler, vivisection_evangelist. Requires CardType::Planeswalker + loyalty ability framework. Very high effort.
- **`forced_attack`** (3): Need `KeywordAbility::AttacksEachCombatIfAble` or equivalent static.
- **`wither`** (1): boggart_ram_gang. Need `KeywordAbility::Wither` variant.
- **`buyback`** (1): searing_touch. Need `KeywordAbility::Buyback`.
- **`dredge`** (1): golgari_grave_troll. Need `AbilityDefinition::Dredge`.

---

## Empty Abilities (`abilities: vec![]`) With No TODO (43 cards)

These cards have NO abilities implemented AND no TODO explaining what's missing. They were likely punted entirely by the author. Many are MDFC/split cards or cards with complex effects.

agadeems_awakening, bala_ged_recovery, barkchannel_pathway, beast_within, blightstep_pathway, bloomvine_regent, boggart_trawler, bottomless_pool, bridgeworks_battle, brightclimb_pathway, call_of_the_nightwing, clearwater_pathway, commit, consign, cragcrown_pathway, darkbore_pathway, decadent_dragon, disciple_of_freyalise, fell_the_profane, funeral_room, generous_gift, hydroelectric_specimen, kabira_takedown, malakir_rebirth, marang_river_regent, memnite, monster_manual, needleverge_pathway, overlord_of_the_hauntwoods, phyrexian_walker, riverglide_pathway, rune_tail_kitsune_ascendant, scavenger_regent, sea_gate_restoration, sejiri_shelter, sink_into_stupor, sundering_eruption, swan_song, turn, turntimber_symbiosis, valakut_awakening, walk_in_closet, witch_enchanter

**Note**: memnite and phyrexian_walker genuinely have no abilities (vanilla creatures). The pathway lands are MDFC backs. Most others need abilities implemented.

---

## Dangerous Partial Implementations (122 cards)

These cards have SOME implementation but produce INCORRECT game state. **Sorted by danger level.**

### CRITICAL: Mana Without Damage (8 cards)

Pain lands that produce colored mana without the self-damage clause. Any test using these lands gets free colored mana.

| Card | Issue |
|------|-------|
| battlefield_forge | {T}: Add {R} or {W} — no 1 damage |
| caves_of_koilos | {T}: Add {W} or {B} — no 1 damage |
| city_of_brass | Any-tap damage trigger missing |
| llanowar_wastes | {T}: Add {B} or {G} — no 1 damage |
| shivan_reef | {T}: Add {U} or {R} — no 1 damage |
| sulfurous_springs | {T}: Add {B} or {R} — no 1 damage |
| underground_river | {T}: Add {U} or {B} — no 1 damage |
| yavimaya_coast | {T}: Add {G} or {U} — no 1 damage |

### CRITICAL: ETB-Tapped Not Set (78 cards)

Lands (and a few permanents) that should enter tapped but don't. This gives players faster mana than intended.

- 56 conditional-ETB-tapped lands (check/fast/bond/reveal/battle/etc.)
- 10 shocklands (should enter tapped unless 2 life paid)
- 12 unconditional enters-tapped (gruul_turf, halimar_depths, etc.)

### HIGH: Approximated Mana Costs (5 cards)

Hybrid mana costs approximated as mono-color. Cards are castable with wrong colors.

| Card | Issue |
|------|-------|
| brokkos_apex_of_forever | {U/B} -> {B} in main + mutate cost |
| connive | {2}{U/B}{U/B} -> {2}{U}{B} |
| nethroi_apex_of_death | {G/W} -> {G} in mutate cost |
| cut_ribbons | X cost approximated as generic 3 |
| mockingbird | {X}{U} — X not representable |

### MEDIUM: Wrong/Broader Target Types (2 cards)

| Card | Issue |
|------|-------|
| putrefy | Uses TargetPermanent instead of TargetArtifactOrCreature |
| vivisection_evangelist | Uses broader target than intended |

### MEDIUM: Activated Abilities Missing Restrictions (10 cards)

Activated abilities implemented without their "activate only if" conditions.

| Card | Issue |
|------|-------|
| arch_of_orazca | Draw ability missing "city's blessing" check |
| bleachbone_verge | Mana ability missing land-control condition |
| cavern_of_souls | Mana ability missing creature-type restriction |
| delighted_halfling | Mana missing uncounterability tracking |
| gloomlake_verge | Mana ability missing land-control condition |
| isolated_chapel | Ability missing condition |
| path_of_ancestry | Scry trigger missing creature-type match |
| tainted_field/isle/wood | Colored mana missing "control a Swamp" check |
| unclaimed_territory | Mana missing creature-type restriction |

### MEDIUM: Static Effects Missing Filters (8 cards)

Static/continuous effects applied too broadly (missing controller/subtype filters).

| Card | Issue |
|------|-------|
| blessed_alliance | SacrificePermanents has no attacking-only filter |
| cascade_bluffs | Filter mana ability missing |
| dragonlord_kolaghan | Trigger filter gap |
| emrakul_the_promised_end | Protection filter gap |
| fervor | No EffectFilter::CreaturesYouControl |
| mystic_remora | Trigger filter gap |
| sunken_ruins | Filter mana ability missing |
| tatyova_steward_of_tides | Flying grant filter gap |

---

## Quick-Win Opportunities (can be fixed NOW with existing DSL)

| Gap | Cards | Effort | Action |
|-----|------:|--------|--------|
| cycling | 5 | Trivial | Add `Cycling { cost: mana_cost!(...) }` to triomes/HQs |
| etb_tapped (simple) | 12 | Trivial | Set `etb_tapped: true` on unconditional lands |
| color_indicator | 1 | Trivial | Set color_indicator on Dryad Arbor |
| flying_missing | 1 | Trivial | Add `Keyword(KeywordAbility::Flying)` to thousand_faced_shadow |
| wither | 1 | Low | Add `KeywordAbility::Wither` variant, add to boggart_ram_gang |
| forced_attack | 3 | Low | Add `KeywordAbility::AttacksEachCombatIfAble` |
| **Total quick wins** | **23** | | |

---

## Implementation Priority Recommendation

### Phase 1: Safety fixes (prevent wrong game state)
1. Set `etb_tapped: true` on 12 simple enters-tapped lands
2. Wire up Cycling on 5 triomes/HQs
3. Add self-damage to 8 pain lands (once mana-with-damage DSL gap is closed)
4. Fix missing Flying on thousand_faced_shadow

### Phase 2: High-impact DSL extensions
1. **Conditional ETB tapped** (56 cards) — replacement effect framework
2. **Sacrifice as cost** (26 cards) — `ActivationCost.sacrifice_self` / `sacrifice_filter`
3. **Targeted activated abilities** (32 cards) — `targets` field on `AbilityDefinition::Activated`
4. **Static grant with filter** (30 cards) — `EffectFilter::CreaturesYouControl`, subtype filters

### Phase 3: Medium-impact DSL extensions
1. Count-based scaling (29 cards)
2. Cost reduction statics (10 cards)
3. Return from zone effects (8 cards)
4. Mana spending restrictions (6 cards)
5. ETB player choice (7 cards)
6. CDA power/toughness (6 cards)

### Phase 4: Specialized mechanics
1. Planeswalker support (4 cards, very high effort)
2. Saga/Class (2 cards, high effort)
3. Replacement effects (11 cards, high effort per card)
4. Land animation (3 cards)
5. Channel (3 cards)
6. Ascend/city's blessing (2 cards)

---

## Uncategorized TODOs (83 cards, 104 lines)

These TODOs didn't match any primary pattern. Many are one-off complex interactions:

- **Can't-be-countered statics**: destiny_spinner, dragonlord_dromoka, rhythm_of_the_wild
- **Devotion-based type changes**: iroas_god_of_victory
- **Conditional hexproof**: dragonlord_ojutai (while untapped), zurgo_and_ojutai (entered this turn)
- **Trigger doubling/suppression statics**: drivnod_carnage_dominus, elesh_norn_mother_of_machines, teysa_karlov
- **Multi-type target filter**: strix_serenade (artifact/creature/planeswalker), krosan_grip, putrefy
- **Counter-spell with payment**: make_disappear ("unless controller pays {2}")
- **Life-threshold statics**: serra_ascendant (30+ life = +5/+5 + flying)
- **Additional combat phases**: karlach_fury_of_avernus
- **Token-origin tracking**: tombstone_stairwell (destroy tokens created by this)
- **ETB with slumber/storage counters**: arixmethes_slumbering_isle
- **Mana type query**: exotic_orchard, reflecting_pool, fellwar_stone
- **Conditional keyword (phase-dependent)**: razorkin_needlehead (first strike on your turn)
- **DFC transform abilities**: frodo_saurons_bane (activated transform + new abilities)
- **Complex ETB**: phyrexian_dreadnought (sacrifice unless sacrifice 12 power)
- **Hidden info**: lumbering_laundry (look at face-down creatures)
- **Stagger mechanic**: lightning_army_of_one
- **Haste-ability acceleration**: thousand_year_elixir, tyvar_jubilant_brawler
- **Block-additional statics**: brave_the_sands
- **Additional land play**: mina_and_denn_wildborn
- **Devotion**: the_world_tree (6+ lands = all lands have all basic types)
