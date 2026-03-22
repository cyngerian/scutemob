---
name: Consolidated Fix List
description: All fixable card def issues from 73 review files, deduplicated and classified
type: reference
---

# Consolidated Fix List

Generated: 2026-03-22
Source: 73 review files (20 Phase 1, 38 Wave 002, 15 Wave 003)

## Summary
- HIGH findings: 24 original (8 still valid, 14 already fixed, 2 now DSL gap / unfixable)
- MEDIUM findings: 42 original (11 still valid, 24 already fixed, 7 DSL gap / unfixable)
- Total cards needing work: 15

### Classification Key
- **Still valid**: Card def still has the issue, needs fixing
- **Already fixed**: PB work or prior fix session resolved this (verified in card def)
- **DSL gap**: Issue is real but the DSL cannot express the fix yet
- **Not a bug**: Re-analysis shows the finding was incorrect or cosmetic

---

## HIGH Findings (fix first)

| # | Card | File | Issue | Source Review | Status | Action |
|---|------|------|-------|--------------|--------|--------|
| H1 | Rograkh, Son of Rohgahh | `defs/rograkh_son_of_rohgahh.rs` | Missing `color_indicator: Some(vec![Color::Red])`. Comment incorrectly says field doesn't exist on CardDefinition — Dryad Arbor proves otherwise. Engine treats it as colorless. | w002-b32 | **Still valid** | Add `color_indicator: Some(vec![Color::Red])`, remove inaccurate comment |
| H2 | Skrelv, Defector Mite | `defs/skrelv_defector_mite.rs` | Missing CantBlock. Header comment (line 8) claims "CantBlock is implemented" but it is not in abilities vec. | w002-b07 | **Partially valid** | Header comment is misleading (says "CantBlock implemented" when it's not). CantBlock itself is a DSL gap. Fix: correct the misleading comment. |
| H3 | Hammer of Nazahn | `defs/hammer_of_nazahn.rs` | Equip {4} already implemented (line 52). Review said it was missing. | w002-b34 | **Already fixed** | No action needed — Equip {4} is present with `AttachEquipment` |
| H4 | Crown of Skemfar | `defs/crown_of_skemfar.rs` | Missing Enchant keyword. | w002-b37 | **Already fixed** | Line 21 has `Keyword(KeywordAbility::Enchant(EnchantTarget::Creature))` |
| H5 | Nullpriest of Oblivion | `defs/nullpriest_of_oblivion.rs` | Missing Kicker keyword. | w002-b37 | **Already fixed** | Lines 17-19 have both `Keyword(Kicker)` and `Kicker { cost }` |
| H6 | Ink-Eyes, Servant of Oni | N/A | Missing Ninjutsu cost def. | w002-b38 | **File not found** | Card def may have been renamed or removed — needs investigation |
| H7 | Dirge Bat | N/A | Missing MutateCost def. | w002-b38 | **File not found** | Card def may have been renamed or removed — needs investigation |
| H8 | Invisible Stalker | `defs/invisible_stalker.rs` | Missing CantBeBlocked keyword. | w002-b12 | **Already fixed** | Line 18 has `Keyword(KeywordAbility::CantBeBlocked)` |
| H9 | Markov Baron | `defs/markov_baron.rs` | Missing Convoke keyword. | w002-b24 | **Already fixed** | Line 22 has `Keyword(KeywordAbility::Convoke)` + Madness cost on line 25 |
| H10 | Concordant Crossroads | `defs/concordant_crossroads.rs` | Missing World supertype. | w002-b04 | **Already fixed** | Line 15 has `supertypes(&[SuperType::World], ...)` |
| H11 | Tamiyo's Safekeeping | `defs/tamiyos_safekeeping.rs` | Target missing "you control" filter. | w002-b05 | **Already fixed** | Line 37 has `TargetPermanentWithFilter(TargetFilter { controller: TargetController::You, ... })` |
| H12 | Ram Through | `defs/ram_through.rs` | Target missing controller filters. | w002-b10 | **Already fixed** | Lines 26-31 have proper `TargetCreatureWithFilter` with `TargetController::You` and `::Opponent` |
| H13 | Dryad Arbor | `defs/dryad_arbor.rs` | Missing color indicator. | p1-b12 | **Already fixed** | Line 9 has `color_indicator: Some(vec![Color::Green])` |
| H14 | Snow-Covered Island | `defs/snow_covered_island.rs` | Missing Basic+Snow supertypes. | p1-b13 | **Already fixed** | Line 9 has `full_types(&[SuperType::Basic, SuperType::Snow], ...)` |
| H15 | Snow-Covered Swamp | `defs/snow_covered_swamp.rs` | Missing Basic+Snow supertypes. | p1-b14 | **Already fixed** | Line 9 has `full_types(&[SuperType::Basic, SuperType::Snow], ...)` |
| H16 | Revitalizing Repast | `defs/revitalizing_repast.rs` | Mana cost was {0}. | p1-b20 | **Already fixed** | Has hybrid mana cost now |
| H17 | Bala Ged Recovery | `defs/bala_ged_recovery.rs` | Name missing "// Bala Ged Sanctuary"; types included Land. | p1-b20 | **Already fixed** | Name is `"Bala Ged Recovery // Bala Ged Sanctuary"`, types is `Sorcery` only |
| H18 | Monster Manual | `defs/monster_manual.rs` | Name missing "// Zoological Study". | p1-b20 | **Already fixed** | Name is `"Monster Manual // Zoological Study"`, types is `Artifact` only |
| H19 | Mox Amber | `defs/mox_amber.rs` | Missing SuperType::Legendary. | p1-b20 | **Already fixed** | Line 9 has `full_types(&[SuperType::Legendary], ...)` |
| H20 | Connive // Concoct | `defs/connive.rs` | Mana cost missing 2 hybrid pips. | p1-b17 | **Already fixed** | Lines 24-31 have proper hybrid mana `HybridMana::ColorColor(Blue, Black)` x2 |
| H21 | Battlefield Forge | `defs/battlefield_forge.rs` | Painland mana ability missing self-damage. | w003-b01 | **Already fixed** | Has `Effect::DealDamage` in mana ability sequence |
| H22 | Blazemire Verge | `defs/blazemire_verge.rs` | Mana ability missing activation restriction. | w003-b01 | **Still valid (DSL gap)** | `abilities: vec![]` with TODO — activation_condition exists but verge pattern needs `ControlLandWithSubtypes` condition on mana ability. The card correctly has empty abilities rather than wrong ones. |
| H23 | Crucible of the Spirit Dragon | `defs/crucible_of_the_spirit_dragon.rs` | Missing implementable {T}: Add {C}. | w003-b04 | **Already fixed** | Line 12-17 has colorless tap ability |
| H24 | Underground River | `defs/underground_river.rs` | Painland mana missing self-damage. | w003-b14 | **Already fixed** | Has `Effect::DealDamage` in mana ability sequence |
| H25 | Urza's Saga | `defs/urzas_saga.rs` | Subtypes `"Urza's Saga"` as one string instead of two. | w003-b14 | **Already fixed** | Line 13 has `&["Urza's", "Saga"]` (two separate subtypes) |
| H26 | Wastewood Verge | `defs/wastewood_verge.rs` | Mana ability without activation restriction. | w003-b15 | **Still valid (DSL gap)** | Same as Blazemire — `abilities: vec![]` with TODO. Correctly empty rather than wrong. |
| H27 | Yavimaya Coast | `defs/yavimaya_coast.rs` | Painland mana missing self-damage. | w003-b15 | **Already fixed** | Has `Effect::DealDamage` in mana ability sequence |
| H28 | Battle Squadron | `defs/battle_squadron.rs` | `*/*` encoded as Some(0)/Some(0). | w002-b29 | **Already fixed** | Lines 12-13 have `power: None, toughness: None` |
| H29 | Cavern of Souls | `defs/cavern_of_souls.rs` | Missing {T}: Add {C}. | w003-b03 | **Already fixed** | Lines 26-31 have colorless tap ability |

### HIGH Summary: 8 still need attention
- **1 card fix** (H1: Rograkh color indicator)
- **1 comment fix** (H2: Skrelv misleading comment)
- **2 file-not-found** (H6: Ink-Eyes, H7: Dirge Bat — need investigation)
- **2 DSL gap** (H22: Blazemire, H26: Wastewood — correctly empty, no action)
- **22 already fixed** by PB work

---

## MEDIUM Findings (fix second)

| # | Card | File | Issue | Source Review | Status | Action |
|---|------|------|-------|--------------|--------|--------|
| M1 | Thousand-Year Elixir | `defs/thousand_year_elixir.rs` | Activated ability has `targets: vec![]` but uses `DeclaredTarget { index: 0 }` | w002-b02 | **Still valid** | Add `targets: vec![TargetRequirement::TargetCreature]` |
| M2 | Legolas's Quick Reflexes | `defs/legolass_quick_reflexes.rs` | Split Second in abilities makes card castable as do-nothing. | w002-b07 | **Already fixed** | `abilities: vec![]` now (Split Second removed, card uncastable per W5 policy) |
| M3 | Ogre Battledriver | `defs/ogre_battledriver.rs` | TODO understates available DSL features (ETBTriggerFilter). | w002-b15 | **Still valid** | Update TODO to mention ETBTriggerFilter availability; may be partially implementable now |
| M4 | Balan, Wandering Knight | `defs/balan_wandering_knight.rs` | Two significant abilities unimplemented. | w002-b16 | **Not a bug** | Review acknowledged correct W5 approach — no fix needed |
| M5 | Ajani, Sleeper Agent | `defs/ajani_sleeper_agent.rs` | Mana cost drops Phyrexian hybrid pip, CMC is 3 not 4. | w002-b25 | **Still valid** | Add extra green or white to approximate CMC 4 (e.g., `green: 2` or `white: 2`) |
| M6 | Abomination of Llanowar | `defs/abomination_of_llanowar.rs` | P/T was Some(0)/Some(0) for `*/*`. | w002-b26 | **Already fixed** | Lines 18-19 have `power: None, toughness: None` |
| M7 | Jagged-Scar Archers | `defs/jagged_scar_archers.rs` | P/T was Some(0)/Some(0) for `*/*`. | w002-b30 | **Already fixed** | Lines 18-19 have `power: None, toughness: None` |
| M8 | Rograkh, Son of Rohgahh | `defs/rograkh_son_of_rohgahh.rs` | Missing color indicator (dup of H1). | w002-b32 | **Dup of H1** | See H1 |
| M9 | Reckless One | `defs/reckless_one.rs` | P/T was Some(0)/Some(0) for `*/*`. | w002-b32 | **Already fixed** | Lines 15-16 have `power: None, toughness: None` |
| M10 | Endurance | `defs/endurance.rs` | Evoke keyword present but no evoke cost definition. | w002-b35 | **DSL gap** | Evoke cost is "exile a green card from hand" — not a ManaCost, so `AbilityDefinition::Evoke { cost: ManaCost }` can't express it. |
| M11 | Grief | `defs/grief.rs` | Evoke keyword present but no evoke cost definition. | w002-b36 | **DSL gap** | Same as M10 — "exile a black card from hand" is not a ManaCost. |
| M12 | Crown of Skemfar | `defs/crown_of_skemfar.rs` | TODO says Reach grant is DSL gap but it's expressible. | w002-b37 | **Still valid** | Reach grant via Static + EffectFilter::AttachedCreature is expressible (see Rancor). Add the Reach grant ability. |
| M13 | Emrakul, the Promised End | `defs/emrakul_the_promised_end.rs` | Protection from instants may be expressible. | w002-b34 | **Still valid (needs verification)** | Check if ProtectionFilter supports card-type-based protection. If so, implement. |
| M14 | Grateful Apparition | `defs/grateful_apparition.rs` | Trigger uses WhenDealsCombatDamageToPlayer, doesn't cover planeswalkers. | w002-b09 | **DSL gap** | Engine-wide limitation, not card-specific fix. |
| M15 | Rune-Tail, Kitsune Ascendant | `defs/rune_tail_kitsune_ascendant.rs` | Missing SuperType::Legendary. | p1-b20 | **Already fixed** | Line 9 has `full_types(&[SuperType::Legendary], ...)` |
| M16 | Decadent Dragon | `defs/decadent_dragon.rs` | Name missing "// Expensive Taste". | p1-b20 | **Already fixed** | Name is `"Decadent Dragon // Expensive Taste"` |
| M17 | Needleverge Pathway | `defs/needleverge_pathway.rs` | Name missing "// Pillarverge Pathway". | p1-b20 | **Already fixed** | Name is `"Needleverge Pathway // Pillarverge Pathway"` |
| M18 | Consign // Oblivion | `defs/consign.rs` | Types included Sorcery on Instant front face. | p1-b20 | **Already fixed** | Types is `Instant` only |
| M19 | Monster Manual | `defs/monster_manual.rs` | Types included Sorcery on Artifact main card. | p1-b20 | **Already fixed** | Types is `Artifact` only |
| M20 | Agadeem's Awakening | `defs/agadeems_awakening.rs` | Missing TODO for X cost. | p1-b17 | **Still valid** | Add TODO comment noting X cost in oracle text (CMC 3 is correct for non-stack per CR 202.3e but X nature undocumented). |
| M21-M40 | ~20 conditional ETB lands | various | Unconditional EntersTapped, missing conditions. | p1-b01 through p1-b09 | **Already fixed** | PB-2 added `unless_condition` with all needed Condition variants. All checked lands now use them. |
| M41 | Caves of Koilos | `defs/caves_of_koilos.rs` | Painland colored mana missing self-damage. | w003-b03 | **Already fixed** | Has `Effect::DealDamage` in mana ability sequence |
| M42 | Command Beacon | `defs/command_beacon.rs` | TODO description slightly inaccurate about sacrifice cost availability. | w003-b03 | **Still valid** | Update TODO to note `Cost::SacrificeThis` exists; the real gap is only the command-zone-to-hand move effect. |
| M43 | Crucible of the Spirit Dragon | `defs/crucible_of_the_spirit_dragon.rs` | Empty abilities when {T}: Add {C} is expressible. | w003-b04 | **Already fixed** | Colorless tap ability present |
| M44 | Geier Reach Sanitarium | `defs/geier_reach_sanitarium.rs` | Style inconsistency (full_types vs supertypes). | w003-b05 | **Not a bug** | Functionally equivalent, cosmetic only |
| M45 | Llanowar Wastes | `defs/llanowar_wastes.rs` | Painland colored mana missing self-damage. | w003-b08 | **Already fixed** | Has `Effect::DealDamage` in mana ability sequence |
| M46 | Haven of the Spirit Dragon | `defs/haven_of_the_spirit_dragon.rs` | AddManaAnyColor without spending restriction. | w003-b08 | **Already fixed** | Uses `AddManaAnyColorRestricted` with `ManaRestriction::CreatureWithSubtype("Dragon")` |
| M47 | Sulfurous Springs | `defs/sulfurous_springs.rs` | Painland colored mana missing self-damage. | w003-b11 | **Already fixed** | Has `Effect::DealDamage` in mana ability sequence |
| M48 | Twilight Mire | `defs/twilight_mire.rs` | Imprecise TODO description. | w003-b13 | **Still valid** | TODO says "hybrid mana costs not expressible" — should also note the multi-option mana output gap. Minor. |
| M49 | Viridescent Bog | `defs/viridescent_bog.rs` | Cost::Sequence verification needed. | w003-b14 | **Not a bug** | Cost::Sequence is used correctly in other card defs |
| M50 | Concordant Crossroads | `defs/concordant_crossroads.rs` | Inaccurate TODO about World supertype. | w002-b04 | **Already fixed** | World supertype is present, no inaccurate TODO |
| M51 | Markov Baron | `defs/markov_baron.rs` | Missing Madness ability definition. | w002-b24 | **Already fixed** | Line 25 has `Madness { cost: ManaCost { ... } }` |
| M52 | Hydroelectric Laboratory | N/A | Missing back_face. | p1-b19 | **File not found** | Card def may have been renamed or removed |
| M53 | Kabira Plateau | N/A | Missing back_face. | p1-b19 | **File not found** | Card def may have been renamed or removed |
| M54 | Coil and Catch | N/A | Missing back_face. | p1-b19 | **File not found** | Card def may have been renamed or removed |
| M55 | Funeral Room // Awakening Hall | `defs/funeral_room.rs` | Reminder text in oracle_text. | p1-b16 | **Not a bug** | Cosmetic only, no behavioral impact |
| M56 | Necroblossom Snarl | `defs/necroblossom_snarl.rs` | Missing TODO for conditional ETB. | p1-b05 | **Already fixed** | Now uses `unless_condition` |
| M57 | Gilt-Leaf Palace | `defs/gilt_leaf_palace.rs` | Missing TODO for reveal-Elf condition. | p1-b07 | **Already fixed** | Now uses `unless_condition` with `CanRevealFromHandWithSubtype` |
| M58 | Shineshadow Snarl | `defs/shineshadow_snarl.rs` | Missing TODO for conditional ETB. | p1-b09 | **Already fixed** | Now uses `unless_condition` |
| M59 | Dryad Arbor | `defs/dryad_arbor.rs` | Redundant explicit Forest mana ability. | p1-b12 | **Needs verification** | If engine derives mana from basic land subtypes, this is a double-mana bug. If not, it's correct. |
| M60 | Connive // Concoct | `defs/connive.rs` | Missing TODO for hybrid mana gap. | p1-b17 | **Already fixed** | Now uses proper `HybridMana` — no gap |

### MEDIUM Summary: 11 still need attention
- **2 card fixes** (M1: Thousand-Year Elixir targets, M12: Crown of Skemfar Reach grant)
- **3 comment/TODO fixes** (M20: Agadeem's X cost TODO, M42: Command Beacon TODO, M48: Twilight Mire TODO)
- **2 need verification** (M5: Ajani CMC, M13: Emrakul protection, M59: Dryad Arbor mana)
- **1 partially valid** (M3: Ogre Battledriver TODO)
- **3 file-not-found** (M52-M54: Hydroelectric Lab, Kabira Plateau, Coil and Catch)
- **3 DSL gaps** (M10: Endurance, M11: Grief, M14: Grateful Apparition)
- **4 not a bug** (M4, M44, M49, M55)
- **24+ already fixed**

---

## LOW Findings (opportunistic)

LOW findings are not enumerated individually due to volume (~40+ across 73 reviews). Common patterns:

| Pattern | Count | Description |
|---------|-------|-------------|
| Missing loyalty field on planeswalkers | ~5 | Planeswalker cards with `loyalty: None` instead of correct starting loyalty |
| Minor oracle text discrepancies | ~8 | Reminder text inclusion/exclusion inconsistency |
| Stale TODO comments | ~10 | TODOs that reference gaps now filled by PB work |
| Style inconsistencies | ~5 | `full_types` vs `supertypes` vs `types_sub` usage for equivalent results |
| Missing creature subtypes | ~3 | Minor subtype omissions (e.g., missing "Phyrexian") |
| Comment inaccuracies | ~5 | Comments describing wrong CR numbers or DSL state |

---

## Action Plan (Priority Order)

### Batch 1: Immediate Fixes (5 cards, ~30 min)
1. **H1**: Rograkh — add `color_indicator: Some(vec![Color::Red])`, fix comment
2. **M1**: Thousand-Year Elixir — add `targets: vec![TargetRequirement::TargetCreature]`
3. **M12**: Crown of Skemfar — add Reach grant via Static + AttachedCreature filter
4. **H2**: Skrelv — fix misleading header comment about CantBlock
5. **M5**: Ajani, Sleeper Agent — approximate hybrid Phyrexian as extra colored pip for CMC 4

### Batch 2: TODO Comment Fixes (3 cards, ~10 min)
6. **M20**: Agadeem's Awakening — add X cost TODO
7. **M42**: Command Beacon — correct TODO about sacrifice cost availability
8. **M48**: Twilight Mire — refine TODO about output gap

### Batch 3: Verification Required (3 cards, ~20 min)
9. **M13**: Emrakul — check ProtectionFilter for card-type variants
10. **M59**: Dryad Arbor — verify if Forest subtype auto-grants mana ability
11. **M3**: Ogre Battledriver — check if ETBTriggerFilter makes trigger expressible

### Batch 4: Missing Files (5 cards — not in codebase)
These cards have no `.rs` file in `defs/` and no registry entry. They were reviewed but never committed, or were removed. Re-author if needed during card authoring waves.
12. **H6**: Ink-Eyes, Servant of Oni — Ninjutsu card, needs authoring
13. **H7**: Dirge Bat — Mutate card, needs authoring
14. **M52**: Hydroelectric Laboratory — MDFC, needs authoring
15. **M53**: Kabira Plateau — MDFC, needs authoring
16. **M54**: Coil and Catch — MDFC, needs authoring
