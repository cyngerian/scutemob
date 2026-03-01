# MTG Engine — Ability Coverage Audit

> Living document. Refresh with `/audit-abilities`.
> Last audited: 2026-03-01 (Melee validated; CR 702.121 confirmed (was incorrectly listed as 702.122 — that is Crew); types.rs:701-711, stack.rs:514-533, stubs.rs:300-302, builder.rs:513-528, abilities.rs:1479-1487+2749-2751, resolution.rs:1888-1897; Wings of the Guard card def; 7 unit tests in melee.rs; game script combat/121; P4 validated 13->14, total validated 105->106)

---

## Status Definitions

| Status | Meaning |
|--------|---------|
| `validated` | Engine implements it, has a card definition using it, has a passing game script |
| `complete` | Engine implements the rules and has unit tests, but no game script yet |
| `partial` | Some aspects work (e.g., static effect works but activation doesn't) |
| `none` | Not implemented |
| `n/a` | Not relevant to Commander or intentionally deferred (Un-sets, digital-only) |

## Priority Definitions

| Priority | Meaning |
|----------|---------|
| P1 | Currently on one of the 54 defined cards, or blocking a `pending_review` script |
| P2 | Top 30 Commander staples (cascade, flashback, equip activation, cycling, etc.) |
| P3 | Commander-relevant but less common |
| P4 | Niche, historical, or set-specific — implement when a card needs it |

---

## Summary

| Priority | Total | Validated | Complete | Partial | None | N/A |
|----------|-------|-----------|----------|---------|------|-----|
| P1       | 42    | 40        | 2        | 0       | 0    | 0   |
| P2       | 17    | 16        | 0        | 0       | 1    | 0   |
| P3       | 40    | 36        | 0        | 0       | 4    | 0   |
| P4       | 100   | 14        | 0        | 0       | 74   | 12  |
| **Total**| **199**| **106**  | **2**    | **0**   | **79**| **12** |

---

## Section 1: Evergreen Keywords

These 16 keywords appear on the most cards and are expected in virtually every Commander game.

| Ability | CR | Priority | Status | Engine File(s) | Card Def | Script | Depends On | Notes |
|---------|----|----------|--------|----------------|----------|--------|------------|-------|
| Deathtouch | 702.2 | P1 | `validated` | `rules/combat.rs` | Multiple creatures | `combat/` scripts | — | Lethal damage = 1 for deathtouch sources |
| Defender | 702.3 | P1 | `validated` | `rules/combat.rs` | Wall of Omens, Arcades | `combat/` scripts | — | Can't attack restriction |
| Double Strike | 702.4 | P1 | `validated` | `rules/combat.rs` | — | `combat/` scripts | — | First strike + normal damage |
| First Strike | 702.7 | P1 | `validated` | `rules/combat.rs` | — | `combat/` scripts | — | Separate first-strike damage step |
| Flash | 702.8 | P1 | `validated` | `rules/casting.rs` | Teferi, Vedalken Orrery | — | — | Cast any time you could cast an instant |
| Flying | 702.9 | P1 | `validated` | `rules/combat.rs` | Multiple creatures | `combat/` scripts | — | Can only be blocked by flying/reach |
| Haste | 702.10 | P1 | `validated` | `rules/combat.rs`, `tests/abilities.rs` | Multiple creatures | `combat/` scripts | — | Ignores summoning sickness |
| Hexproof | 702.11 | P1 | `validated` | `rules/mod.rs`, `rules/protection.rs` | Lightning Greaves, Swiftfoot Boots | — | — | Can't be targeted by opponents |
| Indestructible | 702.12 | P1 | `validated` | `rules/sba.rs` | Avacyn, Darksteel Plate | — | — | Not destroyed by lethal damage or destroy effects |
| Lifelink | 702.15 | P1 | `validated` | `rules/combat.rs` | — | `combat/` scripts | — | Damage dealt = life gained |
| Menace | 702.110 | P1 | `validated` | `rules/combat.rs` | — | `combat/` scripts | — | Must be blocked by 2+ creatures |
| Protection | 702.16 | P1 | `validated` | `rules/protection.rs` | Sword of F&F, Mother of Runes | — | — | DEBT: Damage, Enchanting, Blocking, Targeting |
| Reach | 702.17 | P1 | `validated` | `rules/combat.rs` | — | `combat/` scripts | — | Can block flying creatures |
| Shroud | 702.18 | P1 | `validated` | `rules/mod.rs`, `rules/protection.rs` | Lightning Greaves | — | — | Can't be targeted by anyone |
| Trample | 702.19 | P1 | `validated` | `rules/combat.rs` | — | `combat/` scripts | — | Excess damage to defending player |
| Vigilance | 702.20 | P1 | `validated` | `rules/combat.rs` | — | `combat/` scripts | — | Doesn't tap when attacking |

---

## Section 2: Evasion & Blocking

Additional evasion and blocking-restriction keywords beyond the evergreen set.

| Ability | CR | Priority | Status | Engine File(s) | Card Def | Script | Depends On | Notes |
|---------|----|----------|--------|----------------|----------|--------|------------|-------|
| Ward | 702.21 | P1 | `validated` | `state/types.rs`, `state/builder.rs`, `state/game_object.rs`, `rules/casting.rs`, `rules/abilities.rs`, `rules/events.rs`, `effects/mod.rs` | Adrix and Nev, Twincasters | `stack/055` | — | Ward(u32) enum + trigger via PermanentTargeted/SelfBecomesTargetByOpponent; MayPayOrElse counter-unless-pay; 7 unit tests in `tests/ward.rs`; script `pending_review` |
| Intimidate | 702.13 | P1 | `validated` | `state/types.rs:114`, `rules/combat.rs:453-474` | Bladetusk Boar | `combat/009` | — | CR 702.13b blocking restriction enforced (artifact creature OR shared color); 7 unit tests in `tests/keywords.rs:632`; game script `pending_review` (6/6 assertions pass) |
| Fear | 702.36 | P3 | `validated` | `state/types.rs:385`, `rules/combat.rs:475-489` | Severed Legion | `combat/080` | — | CR 702.36b blocking restriction enforced (artifact creature OR black creature); 7 unit tests in `tests/keywords.rs:1818`; game script approved |
| Shadow | 702.28 | P4 | `validated` | `state/types.rs:572`, `state/hash.rs:450`, `rules/combat.rs:491-502` | Dauthi Slayer | `combat/106` | — | CR 702.28b bidirectional blocking restriction enforced (shadow mismatch = illegal block); 7 unit tests in `tests/shadow.rs`; game script approved |
| Horsemanship | 702.31 | P4 | `validated` | `state/types.rs:600-604`, `state/hash.rs:459-460`, `rules/combat.rs:503-514` | Shu Cavalry | `combat/109` | — | CR 702.31b unidirectional evasion: attacker with horsemanship can't be blocked by creatures without horsemanship; 7 unit tests in `tests/horsemanship.rs`; game script approved |
| Skulk | 702.118 | P4 | `validated` | `state/types.rs:605-608`, `state/hash.rs:461-462`, `rules/combat.rs:520-533` | Furtive Homunculus | `combat/110` | — | CR 702.118b one-directional power-based evasion: blocker_power > attacker_power is illegal (strictly greater than, not >=; equal power CAN block); skulk creature can block anything; 7 unit tests in `tests/skulk.rs`; game script approved |
| Landwalk | 702.14 | P1 | `validated` | `state/types.rs:60-77,133-137`, `rules/combat.rs:484-509` | Bog Raiders | `combat/010` | — | LandwalkType enum (BasicType + Nonbasic variants); KeywordAbility::Landwalk; blocking restriction enforced via calculate_characteristics (handles Blood Moon etc.); 7 unit tests in `tests/keywords.rs:1137-1480`; game script `pending_review` (8/8 assertions pass) |
| CantBeBlocked | 509.1b | P1 | `validated` | `state/types.rs:172`, `state/hash.rs:309`, `rules/combat.rs:441-451`, `cards/definitions.rs:400-437,1386-1395` | Rogue's Passage, Whispersilk Cloak | `combat/014` | — | KeywordAbility::CantBeBlocked enum; blocking restriction enforced in handle_declare_blockers; Rogue's Passage grants via activated ability (UntilEndOfTurn continuous effect, layer 6); Whispersilk Cloak grants via static continuous effect (WhileSourceOnBattlefield); 5 unit tests in `tests/keywords.rs:1510-1784`; 1 card-def test in `tests/card_def_fixes.rs:572`; game script pending_review (4 assertions pass) |

---

## Section 3: Equipment & Attachment

Keywords governing how permanents attach to other permanents.

| Ability | CR | Priority | Status | Engine File(s) | Card Def | Script | Depends On | Notes |
|---------|----|----------|--------|----------------|----------|--------|------------|-------|
| Equip | 702.6 | P1 | `validated` | `state/types.rs:125`, `rules/abilities.rs:118-164`, `effects/mod.rs:1020-1118`, `cards/definitions.rs` | Lightning Greaves, Swiftfoot Boots, Whispersilk Cloak | `layers/012` | — | KeywordAbility::Equip enum; Effect::AttachEquipment with sorcery-speed validation, layer-aware creature type check, activation-time target validation; 14 unit tests in `tests/equip.rs`; game script approved |
| Enchant | 702.5 | P1 | `validated` | `state/types.rs:119-149`, `state/hash.rs:273-283`, `rules/casting.rs:204-241`, `rules/resolution.rs:181-228`, `rules/sba.rs:576-703`, `rules/abilities.rs:728` | Rancor | `stack/062` | — | EnchantTarget enum (Creature/Permanent/Artifact/Enchantment/Land/Planeswalker/Player/CreatureOrPlaneswalker); cast-time target restriction (CR 702.5a/303.4a); Aura attachment on resolution (CR 303.4b, AuraAttached event); SBA 704.5m type-mismatch + unattached + self-enchantment (CR 303.4d); AuraFellOff trigger wiring for WhenDies; 11 unit tests in `tests/enchant.rs`; game script pending_review (19/19 assertions pass) |
| Bestow | 702.103 | P3 | `validated` | `state/types.rs:347`, `state/game_object.rs:323-333`, `state/stack.rs:66-75`, `state/mod.rs:281-356`, `state/hash.rs:380-381,532-533,1156-1157,1787-1788,2414-2415`, `cards/card_definition.rs:176-183`, `rules/casting.rs:62,184-223,497-505,728-736`, `rules/command.rs:106-111`, `rules/engine.rs:79,94`, `rules/resolution.rs:49-58,219-229`, `rules/sba.rs:709-745`, `rules/events.rs:266-269`, `rules/copy.rs:179-180`, `testing/replay_harness.rs:290-304` | Boon Satyr | `layers/081` | Enchant | KeywordAbility::Bestow enum; AbilityDefinition::Bestow{cost}; `is_bestowed` on permanents, `was_bestowed` on stack objects; alt cost validation (CR 118.9a: no combine with flashback/evoke); type transformation on cast (becomes Aura + enchant creature); target-illegal fallback to creature (CR 702.103e/608.3b); SBA 702.103f bestowed Aura revert (exception to 704.5m); copy preserves bestow (CR 702.103c); zone-change resets (CR 400.7); `cast_spell_bestow` harness action; 9 unit tests in `tests/bestow.rs`; script `pending_review` |
| Reconfigure | 702.151 | P4 | `none` | — | — | — | Equip | Artifact creature that can attach/detach |
| Fortify | 702.67 | P4 | `none` | — | — | — | — | Equip for lands (Fortifications) |
| Living Weapon | 702.92 | P3 | `validated` | `state/types.rs:353-360`, `state/builder.rs:582-616`, `state/hash.rs:384-385`, `cards/card_definition.rs:418-426`, `effects/mod.rs:348-393` | Batterskull | `stack/082` | Equip | KeywordAbility::LivingWeapon enum; ETB trigger wired in builder.rs (SelfEntersBattlefield -> CreateTokenAndAttachSource); atomic token creation + Equipment attachment before SBAs (CR 702.92a); Germ is 0/0 black Phyrexian/Germ creature token; Batterskull card def (definitions.rs:1511); 6 unit tests in `tests/living_weapon.rs` (ETB trigger fires, Germ characteristics, buff survival, zero-toughness SBA, equip-to-other, multiplayer single trigger); script `pending_review` |

---

## Section 4: Alternative Casting

Keywords that allow spells to be cast from non-hand zones or at alternate costs.

| Ability | CR | Priority | Status | Engine File(s) | Card Def | Script | Depends On | Notes |
|---------|----|----------|--------|----------------|----------|--------|------------|-------|
| Flashback | 702.34 | P2 | `validated` | `rules/casting.rs`, `rules/resolution.rs`, `state/stack.rs` | Think Twice, Faithless Looting | `stack/060` | — | Cast from graveyard paying alternative cost; exiled on any stack departure (CR 702.34a) |
| Madness | 702.35 | P3 | `validated` | `state/types.rs`, `cards/card_definition.rs`, `state/stack.rs`, `state/stubs.rs`, `effects/mod.rs`, `rules/casting.rs`, `rules/turn_actions.rs`, `rules/abilities.rs`, `rules/resolution.rs`, `testing/replay_harness.rs` | Fiery Temper | `stack/083` | — | Discard replacement exiles card (CR 702.35a); MadnessTrigger on stack; cast from exile paying madness cost (CR 702.35b); sorcery timing bypass; decline puts card in graveyard; 11 unit tests in madness.rs |
| Miracle | 702.94 | P3 | `validated` | `state/types.rs`, `cards/card_definition.rs`, `state/stack.rs`, `state/stubs.rs`, `state/player.rs`, `rules/miracle.rs`, `rules/casting.rs`, `rules/engine.rs`, `rules/turn_actions.rs`, `rules/replacement.rs`, `rules/resolution.rs`, `rules/abilities.rs`, `effects/mod.rs`, `testing/replay_harness.rs` | Terminus | `stack/084` | — | Draw-site detection (first draw this turn), MiracleRevealChoiceRequired event, ChooseMiracle command, MiracleTrigger on stack, alternative cost in casting.rs (mutual exclusion with flashback/evoke/bestow/madness), sorcery timing bypass, MiracleTrigger no-op resolution, cards_drawn_this_turn reset at turn boundary for all players; 11 unit tests in miracle.rs |
| Escape | 702.138 | P3 | `validated` | `state/types.rs`, `cards/card_definition.rs`, `rules/casting.rs`, `rules/resolution.rs`, `testing/replay_harness.rs` | Ox of Agonas | `stack/085` | — | KeywordAbility::Escape enum + AbilityDefinition::Escape{cost,exile_count} + EscapeWithCounter; graveyard zone detection, exile cost payment, mutual exclusion with all other alt costs, sorcery timing bypass, was_escaped propagation to permanent, +1/+1 counter on ETB; 16 unit tests in escape.rs |
| Foretell | 702.143 | P3 | `validated` | `rules/foretell.rs`, `rules/casting.rs`, `state/types.rs`, `cards/card_definition.rs`, `state/game_object.rs`, `state/stack.rs`, `rules/engine.rs`, `testing/replay_harness.rs` | Saw It Coming | `stack/086` | — | KeywordAbility::Foretell enum (disc 51) + AbilityDefinition::Foretell{cost}; is_foretold/foretold_turn on GameObject; Command::ForetellCard special action; GameEvent::CardForetold; foretell.rs: priority check, turn check, {2} payment, exile face-down (CR 702.143b no stack); casting.rs: foretell zone detection, same-turn restriction, foretell cost, mutual exclusion with all other alt costs (CR 118.9a); 18 unit tests in foretell.rs |
| Retrace | 702.81 | P4 | `none` | — | — | — | — | Cast from graveyard by discarding a land |
| Jump-Start | 702.133 | P4 | `none` | — | — | — | — | Cast from graveyard by discarding a card |
| Aftermath | 702.127 | P4 | `none` | — | — | — | — | Cast second half from graveyard only |
| Disturb | 702.146 | P4 | `none` | — | — | — | — | Cast transformed from graveyard |
| Unearth | 702.84 | P3 | `validated` | `rules/abilities.rs`, `rules/turn_actions.rs`, `rules/replacement.rs`, `rules/resolution.rs`, `state/types.rs`, `state/stack.rs`, `cards/card_definition.rs` | Dregscape Zombie | `stack/087` | — | Full flow: activate from graveyard (sorcery speed), return to battlefield w/ haste, exile at end step delayed trigger, zone-change replacement (leave BF -> exile). 12 unit tests in `unearth.rs` |
| Embalm | 702.128 | P4 | `none` | — | — | — | — | Create token copy from graveyard, white, no mana cost |
| Eternalize | 702.129 | P4 | `none` | — | — | — | — | Create 4/4 token copy from graveyard, black |
| Ninjutsu | 702.49 | P4 | `none` | — | — | — | — | Swap unblocked attacker for creature in hand |
| Plot | 702.164 | P4 | `none` | — | — | — | — | Exile from hand, cast for free on a later turn |
| Blitz | 702.152 | P4 | `none` | — | — | — | — | Alternative cost, gains haste + "draw when dies" + sacrifice at end |
| Dash | 702.109 | P4 | `none` | — | — | — | — | Alternative cost, gains haste, return to hand at end |

---

## Section 5: Cost Modification

Keywords that change how mana costs are paid.

| Ability | CR | Priority | Status | Engine File(s) | Card Def | Script | Depends On | Notes |
|---------|----|----------|--------|----------------|----------|--------|------------|-------|
| Convoke | 702.51 | P2 | `validated` | `state/types.rs:237`, `state/hash.rs:342`, `rules/command.rs:71`, `rules/casting.rs:499-620`, `rules/engine.rs:74-80`, `testing/replay_harness.rs:220-229`, `testing/script_schema.rs:222` | Siege Wurm | `stack/063` | — | KeywordAbility::Convoke enum; convoke_creatures field on CastSpell command; apply_convoke_reduction validates creatures (battlefield, controlled, untapped, creature type, no duplicates), reduces colored then generic mana, taps creatures, emits PermanentTapped (CR 702.51a/b/d); harness resolves creature names to ObjectIds; 12 unit tests in `tests/convoke.rs`; game script pending_review (all assertions pass) |
| Delve | 702.66 | P2 | `validated` | `state/types.rs:243`, `state/hash.rs:344`, `rules/command.rs:80`, `rules/casting.rs:278-318,659-714`, `testing/replay_harness.rs:203-236`, `testing/script_schema.rs:223-227` | Treasure Cruise | `stack/064` | — | KeywordAbility::Delve enum; delve_cards field on CastSpell command; apply_delve_reduction validates graveyard membership, no duplicates, count <= generic, exiles cards, emits ObjectExiled (CR 702.66a/b); harness resolves card names from graveyard; 10 unit tests in `tests/delve.rs`; game script pending_review (assertions pass) |
| Improvise | 702.126 | P3 | `validated` | state/types.rs:333, rules/casting.rs:360-378+804-900, rules/command.rs:72, rules/engine.rs, state/hash.rs:378 | Reverse Engineer | stack/079 | Tap artifacts to pay generic mana; 12 unit tests in improvise.rs |
| Affinity | 702.41 | P3 | `validated` | state/types.rs:144+363, rules/casting.rs:695+1917-1982, state/hash.rs:288-292+408-409 | Frogmite | stack/088 | AffinityTarget enum {Artifacts, BasicLandType}; apply_affinity_reduction + count_affinity_permanents + matches_affinity_target in casting.rs; 12 unit tests in affinity.rs; script pending_review |
| Undaunted | 702.125 | P3 | `validated` | `state/types.rs:364,371`, `state/hash.rs:413-414`, `rules/casting.rs:701+2007-2030`, `cards/definitions.rs:2709-2721` | Sublime Exhalation | `stack/089` | — | KeywordAbility::Undaunted discriminant 54; apply_undaunted_reduction counts live opponents (CR 702.125b); floors generic at 0 (CR 601.2f); multiplayer scaling tested (2-, 4-, 6-player); commander tax + Undaunted composition; Affinity + Undaunted stack; 11 unit tests in `tests/undaunted.rs`; script `pending_review` |
| Assist | 702.132 | P4 | `none` | — | — | — | — | Another player may pay generic mana costs |
| Surge | 702.117 | P4 | `none` | — | — | — | — | Alternative cost if you or teammate cast a spell this turn |
| Spectacle | 702.137 | P4 | `none` | — | — | — | — | Alternative cost if opponent lost life this turn |
| Emerge | 702.119 | P4 | `none` | — | — | — | — | Alternative cost by sacrificing a creature |
| Bargain | 702.166 | P4 | `none` | — | — | — | — | Additional cost: sacrifice an artifact/enchantment/token |

---

## Section 6: Spell & Ability Modifiers

Keywords that modify how spells are cast, copied, or resolved.

| Ability | CR | Priority | Status | Engine File(s) | Card Def | Script | Depends On | Notes |
|---------|----|----------|--------|----------------|----------|--------|------------|-------|
| Storm | 702.40 | P1 | `validated` | `rules/casting.rs`, `rules/copy.rs` | Grapeshot (etc.) | `stack/` scripts | — | Copy for each prior spell this turn |
| Cascade | 702.85 | P1 | `validated` | `rules/casting.rs`, `rules/copy.rs` | Bloodbraid Elf (etc.) | `stack/` scripts | — | Exile until nonland with lesser MV, cast free |
| Kicker | 702.33 | P2 | `validated` | `state/types.rs:244`, `cards/card_definition.rs:152-166`, `effects/mod.rs:60-91,1744-1745`, `state/stack.rs:58`, `state/game_object.rs:269`, `rules/command.rs:88`, `rules/casting.rs:169-205,393,549-568`, `rules/resolution.rs:149-156,187`, `testing/script_schema.rs:228-232`, `testing/replay_harness.rs:204,238` | Burst Lightning, Torch Slinger | `stack/065` | — | KeywordAbility::Kicker enum + AbilityDefinition::Kicker { cost, is_multikicker }; Condition::WasKicked; kicker_times_paid on StackObject + GameObject; kicker_times on CastSpell; get_kicker_cost + validation/payment in casting.rs; kicker propagation to EffectContext in resolution.rs; harness kicked:bool support; 10 unit tests in `tests/kicker.rs`; game script pending_review (assertions pass) |
| Overload | 702.96 | P3 | `validated` | `state/types.rs:599`, `cards/card_definition.rs:255-263,821`, `state/stack.rs:127-134`, `effects/mod.rs:66-69,2759-2760`, `rules/command.rs:157-166`, `rules/casting.rs:485-529,591-597,737-743,974-975`, `rules/resolution.rs:179-186`, `testing/replay_harness.rs:668-689` | Vandalblast | `baseline/108` | — | KeywordAbility::Overload enum + AbilityDefinition::Overload { cost }; Condition::WasOverloaded; cast_with_overload on CastSpell command; was_overloaded on StackObject + EffectContext; overload cost payment as alternative cost (CR 118.9); no-targets enforcement (CR 702.96b); alternative cost mutual exclusion (flashback/evoke/bestow/madness/miracle/escape/foretell); commander tax stacking; harness cast_spell_overload action; 11 unit tests in `tests/overload.rs`; game script baseline/108 approved |
| Replicate | 702.56 | P4 | `none` | — | — | — | — | Pay replicate cost N times → N copies |
| Splice | 702.47 | P4 | `none` | — | — | — | — | Reveal from hand, add text to another spell |
| Entwine | 702.42 | P4 | `none` | — | — | — | — | Pay entwine cost to choose all modes |
| Fuse | 702.102 | P4 | `none` | — | — | — | — | Cast both halves of a split card |
| Buyback | 702.27 | P3 | `validated` | `state/types.rs:497-503`, `state/stack.rs:118`, `rules/casting.rs:67,597-625,894,1113-1129`, `rules/resolution.rs:489-490` | Searing Touch | `stack/094` | — | KeywordAbility::Buyback enum + AbilityDefinition::Buyback { cost }; cast_with_buyback param on CastSpell command; get_buyback_cost lookup in casting.rs; was_buyback_paid on StackObject; resolution destination check in resolution.rs (CR 702.27a); flashback exile overrides buyback (CR 702.34a); 9 unit tests in `tests/buyback.rs` covering basic payment, cost aggregation, counter interaction, fizzle behavior; game script stack/094 pending_review (assertions pass) |
| Spree | 702.165 | P4 | `none` | — | — | — | — | Choose modes, pay cost for each |
| Cleave | 702.148 | P4 | `none` | — | — | — | — | Pay cleave cost → remove bracketed text |
| Escalate | 702.121 | P4 | `none` | — | — | — | — | Pay escalate cost for each mode beyond the first |
| Split Second | 702.61 | P2 | `validated` | `state/types.rs:250-255`, `state/hash.rs:347-348`, `rules/casting.rs:66-72,1060-1074`, `rules/abilities.rs:63-70,388-395` | Krosan Grip | `stack/066` | — | KeywordAbility::SplitSecond enum + has_split_second_on_stack helper; CastSpell gate (CR 702.61a), ActivateAbility gate (CR 702.61a), CycleCard gate (CR 702.61a); mana abilities exempt (CR 702.61b); triggered abilities still fire (CR 702.61b); uses calculate_characteristics for layer-aware keyword check; 8 unit tests in `tests/split_second.rs`; game script pending_review |
| Gravestorm | 702.69 | P4 | `none` | — | — | — | Storm | Copy for each permanent put into graveyard this turn |

---

## Section 7: Combat Triggers & Modifiers

Keywords that modify combat or trigger during combat.

| Ability | CR | Priority | Status | Engine File(s) | Card Def | Script | Depends On | Notes |
|---------|----|----------|--------|----------------|----------|--------|------------|-------|
| Flanking | 702.25 | P4 | `validated` | `state/types.rs:639-643`, `rules/abilities.rs:1468-1556`, `rules/resolution.rs:1675-1681`, `rules/combat.rs:646-660` | Suq'Ata Lancer (`defs/suq_ata_lancer.rs`) | `combat/114` | — | KeywordAbility::Flanking enum; triggered ability fires at declare blockers (CR 702.25a); blocker without flanking gets -1/-1 until EOT; multiple instances trigger separately (CR 702.25b); FlankingTrigger stack kind; flanking_blocker_id on PendingTrigger; 7 unit tests in `tests/flanking.rs`; game script combat/114 validated |
| Bushido | 702.45 | P4 | `validated` | `state/types.rs:644-648`, `state/builder.rs:687-722`, `rules/abilities.rs:1561-1578` | Devoted Retainer (`defs/devoted_retainer.rs`) | `combat/115` | — | KeywordAbility::Bushido(N) enum; two TriggeredAbilityDefs per instance via builder (SelfBlocks + SelfBecomesBlocked); ApplyContinuousEffect with ModifyBoth(+N) until EOT; SelfBecomesBlocked dispatch in abilities.rs fires once per attacker (CR 509.1h); multiple instances trigger separately (CR 702.45b); 7 unit tests in `tests/bushido.rs`; game script combat/115 validated |
| Provoke | 702.39 | P4 | `validated` | `state/types.rs:660-673`, `state/stack.rs:478-491`, `state/combat.rs:58-64`, `state/builder.rs:745-761`, `rules/abilities.rs:1409-1452+2311-2530`, `rules/resolution.rs:1788-1830`, `rules/combat.rs:630-797` | Goblin Grappler (`defs/goblin_grappler.rs`) | `combat/117` | — | KeywordAbility::Provoke enum; triggered ability fires at declare attackers (CR 702.39a); ProvokeTrigger stack kind with source_object + provoked_creature; untaps provoked creature on resolution; adds forced_blocks entry to CombatState (CR 509.1c); forced-block enforcement in combat.rs checks evasion/restrictions before requiring block; multiple instances trigger separately targeting different creatures (CR 702.39b); no trigger when no valid target (CR 603.3d); multiplayer-correct (targets only defending player's creatures, CR 508.5a); 7 unit tests in `tests/provoke.rs`; game script combat/117 validated |
| Exalted | 702.83 | P2 | `validated` | `state/types.rs:256`, `state/hash.rs:349+904+946`, `state/game_object.rs:146`, `state/stubs.rs:74`, `state/builder.rs:396-420`, `rules/abilities.rs:667-697,983-989` | Akrasan Squire | `combat/067` | — | KeywordAbility::Exalted enum + TriggerEvent::ControllerCreatureAttacksAlone; exalted_attacker_id on PendingTrigger; builder keyword-to-trigger translation; check_triggers attacks-alone detection + flush_pending_triggers Target::Object wiring; 8 unit tests in `tests/exalted.rs`; game script pending_review |
| Battle Cry | 702.91 | P3 | `validated` | `state/types.rs:309`, `state/hash.rs:369-370+1976`, `cards/card_definition.rs:750-753`, `state/builder.rs:438-451`, `effects/mod.rs:1995-1998`, `tools/replay-viewer/src/view_model.rs:608` | Signal Pest | `combat/076` | — | KeywordAbility::BattleCry enum + ForEachTarget::EachOtherAttackingCreature; builder keyword-to-trigger translation (WhenAttacks + ForEach over EachOtherAttackingCreature with +1/+0); effects collect_for_each arm excludes source; 7 unit tests in `tests/battle_cry.rs`; Signal Pest card def in definitions.rs:1866; game script combat/076 validated |
| Myriad | 702.116 | P3 | `validated` | state/types.rs, state/stack.rs, state/stubs.rs, state/builder.rs, state/game_object.rs, rules/abilities.rs, rules/resolution.rs, rules/turn_actions.rs | Warchief Giant | myriad.rs (7) | combat/093 | Token copies attacking each opponent; end-of-combat exile; myriad_exile_at_eoc flag; multiple instances trigger separately (CR 702.116b) |
| Melee | 702.121 | P4 | `validated` | `state/types.rs:701-711`, `state/stack.rs:514-533`, `state/stubs.rs:300-302`, `state/builder.rs:513-528`, `state/hash.rs:495-496+1466-1467`, `rules/abilities.rs:1479-1487+2749-2751`, `rules/resolution.rs:1888-1897` | Wings of the Guard (`defs/wings_of_the_guard.rs`) | `combat/121` | — | KeywordAbility::Melee enum (discriminant 83); StackObjectKind::MeleeTrigger (discriminant 23) with source_object; builder generates TriggeredAbilityDef(SelfAttacks) with custom MeleeTrigger resolution; resolution.rs counts distinct opponents attacked with at least one creature this combat and applies +N/+N until EOT; CR 702.121b multiple instances trigger separately; planeswalker attacks do not count opponents (Ruling 2016-08-23); source-leaves-battlefield = no bonus; 7 unit tests in `tests/melee.rs` (basic-1-opp, multiplayer-2-opps, multiplayer-3-opps, planeswalker-no-count, multiple-instances, source-leaves, attacking-alone); card def wings_of_the_guard.rs; game script combat/121 validated |
| Enlist | 702.155 | P4 | `none` | — | — | — | — | Tap non-attacking creature to add its power |
| Annihilator | 702.86 | P2 | `validated` | `state/types.rs:269`, `state/hash.rs:351+2223`, `state/stubs.rs:83`, `state/builder.rs:418-435`, `rules/abilities.rs:657-677,1000-1003`, `cards/card_definition.rs:349-354`, `effects/mod.rs:1034` | Ulamog's Crusher | `combat/068` | — | KeywordAbility::Annihilator(u32) enum + Effect::SacrificePermanents; defending_player_id on PendingTrigger; builder keyword-to-trigger translation (WhenAttacks + SacrificePermanents); check_triggers dispatch + flush_pending_triggers Target::Player wiring; 8 unit tests in `tests/annihilator.rs`; game script pending_review; TODO: "attacks each combat if able" static ability on Ulamog's Crusher is cosmetic only |
| Dethrone | 702.105 | P3 | `validated` | `state/types.rs:372`, `state/game_object.rs:179`, `state/builder.rs:464`, `rules/abilities.rs:965` | Marchesa's Emissary | `tests/dethrone.rs` (8 tests) | `combat/081` | Life-total comparison; planeswalker exclusion; eliminated-player exclusion |
| Rampage | 702.23 | P4 | `validated` | `state/types.rs:649-659`, `state/stack.rs:448-468`, `state/stubs.rs:256-268`, `state/hash.rs:476-477+1108-1110+1411-1418`, `state/builder.rs:724-743`, `rules/abilities.rs:1596-1616+2425-2431`, `rules/resolution.rs:1723-1743` | Wolverine Pack | `rampage.rs` (8 tests) | `combat/116` | KeywordAbility::Rampage(u32) enum; StackObjectKind::RampageTrigger with source_object+rampage_n; builder generates TriggeredAbilityDef(SelfBecomesBlocked) per Rampage instance; abilities.rs tags is_rampage_trigger+rampage_n on PendingTrigger at BlockersDeclared; resolution.rs computes bonus=(blocker_count-1)*N as +N/+N until EOT; CR 702.23c multiple instances trigger separately; 8 unit tests (blocked-by-2, blocked-by-1-no-bonus, blocked-by-3-scaled, multiple-instances, not-blocked, bonus-expires-EOT, bonus-at-resolution, rampage-3-by-4); card def wolverine_pack.rs; game script combat/116 |
| Banding | 702.22 | P4 | `n/a` | — | — | — | — | Extremely complex, rarely used; intentionally deferred |
| Renown | 702.112 | P4 | `validated` | `state/types.rs:683-692`, `state/game_object.rs:437-444`, `state/hash.rs:488-492+658-659+1449-1456`, `state/stack.rs:492-513`, `state/stubs.rs:287-298`, `rules/abilities.rs:2137-2178+2664-2673`, `rules/resolution.rs:1842-1874` | Topan Freeblade | `combat/119` | — | KeywordAbility::Renown(u32) enum (discriminant 81); `is_renowned` designation on GameObject (CR 702.112b); StackObjectKind::RenownTrigger (discriminant 22) with source_object+renown_n; CombatDamageDealt dispatch with intervening-if at trigger time (CR 603.4); resolution re-checks intervening-if + places N +1/+1 counters + sets is_renowned; CR 702.112c multiple instances trigger separately; CR 400.7 resets on zone change; Ruling 2015-06-22 source-left-battlefield; 7 unit tests in `tests/renown.rs`; card def topan_freeblade.rs; game script combat/119 pending_review |
| Afflict | 702.130 | P4 | `validated` | `state/types.rs:674-682`, `state/hash.rs:483-486`, `state/builder.rs:764-782`, `rules/abilities.rs:1693-1703` | Khenra Eternal | `combat/118` | — | KeywordAbility::Afflict(u32) enum (discriminant 80); TriggeredAbilityDef via builder.rs using SelfBecomesBlocked + LoseLife with DeclaredTarget; defending_player_id tagging in abilities.rs BlockersDeclared handler (CR 508.5 multiplayer); 6 unit tests in `tests/afflict.rs` (basic life loss, not-blocked-no-trigger, multiple-blockers-single-trigger, multiple-instances-trigger-separately, multiplayer-correct-defending-player, life-loss-not-damage); card def khenra_eternal.rs; game script combat/118 |

---

## Section 8: Creature Enters/Leaves/Dies

Keywords triggered by creatures entering, leaving, or dying.

| Ability | CR | Priority | Status | Engine File(s) | Card Def | Script | Depends On | Notes |
|---------|----|----------|--------|----------------|----------|--------|------------|-------|
| Persist | 702.79 | P2 | `validated` | `state/types.rs:277`, `state/game_object.rs:154-163`, `state/hash.rs:357`, `state/builder.rs:438-470`, `rules/events.rs:249`, `rules/abilities.rs:778-784,1175-1191`, `rules/sba.rs:301-356`, `rules/replacement.rs:754-769,1011-1039`, `effects/mod.rs:325-426,762-767,1071-1161` | Kitchen Finks | `combat/069` | — | KeywordAbility::Persist enum; InterveningIf::SourceHadNoCounterOfType(MinusOneMinusOne) in game_object.rs; pre_death_counters field on CreatureDied event (7 emission sites in sba.rs, effects/mod.rs, replacement.rs); check_intervening_if extended in abilities.rs; builder keyword-to-trigger translation (SelfDies + intervening-if + Sequence(MoveZone, AddCounter)); ctx.source update after MoveZone in effects/mod.rs:762-767; 6 unit tests in `tests/persist.rs`; game script pending_review |
| Undying | 702.93 | P2 | `validated` | `state/types.rs:278-285`, `state/hash.rs:358-359`, `state/builder.rs:471-498`, `rules/abilities.rs:778-784,1175-1191`, `effects/mod.rs:325-426,762-767` | Young Wolf | `combat/070` | — | KeywordAbility::Undying enum; InterveningIf::SourceHadNoCounterOfType(PlusOnePlusOne) in game_object.rs; pre_death_counters on CreatureDied event (8 emission sites); builder keyword-to-trigger translation (SelfDies + intervening-if + Sequence(MoveZone, AddCounter)); ctx.source update after MoveZone in effects/mod.rs; 6 unit tests in `tests/undying.rs`; game script pending_review |
| Riot | 702.136 | P3 | `validated` | `state/types.rs:456` (KeywordAbility::Riot discriminant 56), `state/hash.rs:416`, `tools/replay-viewer/src/view_model.rs:638`, `rules/resolution.rs:256-292` (inline ETB replacement, counts from card def to handle OrdSet deduplication, default choice: +1/+1 counter; TODO Command::ChooseRiot for interactive choice) | Zhur-Taa Goblin | `combat/082` (pending_review) | — | CR 702.136a fully enforced as replacement effect (CR 614.1c); OrdSet deduplication handled by counting Riot from card definition abilities list; CR 702.136b (multiple instances) noted in impl; 5 unit tests in `tests/riot.rs` |
| Afterlife | 702.135 | P3 | `validated` | `state/types.rs:317-324` (KeywordAbility::Afterlife(u32)), `state/hash.rs:371-372`, `state/builder.rs:530-540` (trigger generation: SelfDies + CreateToken Spirit), `tools/replay-viewer/src/view_model.rs:609` | Ministrant of Obligation | `combat/077` | — | CR 702.135a fully enforced; Afterlife N creates N 1/1 white/black Spirit tokens with flying on death; builder keyword-to-trigger translation (SelfDies + CreateToken); no intervening-if (unlike Persist/Undying); multiple instances trigger separately (CR 702.135b); token-with-Afterlife edge case tested; multiplayer APNAP ordering; 6 unit tests in `tests/afterlife.rs`; game script validated |
| Exploit | 702.110 | P3 | `validated` | `state/types.rs:466` (KeywordAbility::Exploit discriminant 57), `state/hash.rs:419`, `tools/replay-viewer/src/view_model.rs:639`, `state/stubs.rs:133` (PendingTrigger.is_exploit_trigger), `state/hash.rs:1003`, `state/stack.rs:241` (StackObjectKind::ExploitTrigger discriminant 10), `state/hash.rs:1205`, `rules/abilities.rs` (PermanentEnteredBattlefield arm, after Evoke block; flush arm), `rules/resolution.rs` (ExploitTrigger arm; default: decline sacrifice; fizzle arm) | Qarsi Sadist (`qarsi-sadist`) | `stack/090` (pending_review) | — | CR 702.110a fully enforced; ETB triggered ability; player may sacrifice a creature on resolution; default = decline; fizzle arm handles empty choice; 5 unit tests in `tests/exploit.rs`; game script pending_review |
| Evoke | 702.74 | P2 | `validated` | `state/types.rs:293-301` (KeywordAbility::Evoke), `cards/card_definition.rs:168-175` (AbilityDefinition::Evoke { cost }), `state/stack.rs:59-65` (was_evoked on StackObject), `state/game_object.rs:311-317` (was_evoked on GameObject), `rules/casting.rs:56-632` (alternative cost handling, get_evoke_cost, flashback conflict), `rules/abilities.rs:568-587+1082-1103` (evoke sacrifice PendingTrigger + EvokeSacrificeTrigger kind), `rules/resolution.rs:185-190+452-458` (was_evoked transfer + EvokeSacrificeTrigger resolution), `state/hash.rs:362+510+927+1086+1131+2371` | Mulldrifter | `stack/074` | — | CR 702.74a fully enforced; evoke as alternative cost (CR 118.9); was_evoked flag on StackObject + GameObject; sacrifice trigger via PendingTrigger with is_evoke_sacrifice; EvokeSacrificeTrigger stack kind; flashback conflict rejected (CR 118.9a: only one alternative cost); commander tax applies on top of evoke cost; blink fizzles sacrifice trigger (new object); 8 unit tests in `tests/evoke.rs`; game script validated (all assertions pass) |
| Encore | 702.141 | P4 | `none` | — | — | — | — | Exile from graveyard → token copy for each opponent, attack, sacrifice at end |
| Champion | 702.72 | P4 | `none` | — | — | — | — | ETB exile a creature you control; leaves → return it |
| Devour | 702.82 | P4 | `none` | — | — | — | — | ETB: sacrifice creatures for +1/+1 counters |
| Tribute | 702.107 | P4 | `none` | — | — | — | — | Opponent chooses: +1/+1 counters or ability triggers |
| Fabricate | 702.123 | P4 | `none` | — | — | — | — | ETB: choose +1/+1 counters or create Servo tokens |
| Decayed | 702.147 | P4 | `validated` | state/types.rs:616-628, state/game_object.rs:399-408, rules/combat.rs:285-300+396-401, rules/turn_actions.rs:606-632 | Shambling Ghast | decayed.rs (8 tests) | baseline/112 | Can't block; sacrifice at end of combat when it attacks. Flag-on-object pattern (like Myriad); persists even if keyword removed (ruling 2021-09-24). |
| Training | 702.149 | P4 | `validated` | state/types.rs:693-700 (KeywordAbility::Training, discriminant 82), state/game_object.rs:206-211 (TriggerEvent::SelfAttacksWithGreaterPowerAlly, discriminant 19), state/builder.rs:489-511 (TriggeredAbilityDef auto-generation), state/hash.rs:493-494+1189-1190, rules/abilities.rs:1508-1550 (AttackersDeclared dispatch: layer-aware power comparison, SelfAttacksWithGreaterPowerAlly event) | Gryff Rider | training.rs (7 tests) | combat/120 | CR 702.149a+b fully enforced; triggered ability auto-generated from keyword in builder.rs; AttackersDeclared handler checks strictly-greater power among co-attackers (layer-aware); multiple instances trigger separately (CR 702.149b); 7 unit tests cover basic trigger, attacking alone (negative), equal power (negative), lower power (negative), multiple instances, two training creatures, 4-player multiplayer; game script approved |
| Backup | 702.160 | P4 | `none` | — | — | — | — | ETB: put +1/+1 counters on target creature, it gains abilities |

---

## Section 9: Counters & Growth

Keywords involving counter manipulation and creature growth.

| Ability | CR | Priority | Status | Engine File(s) | Card Def | Script | Depends On | Notes |
|---------|----|----------|--------|----------------|----------|--------|------------|-------|
| Modular | 702.43 | P3 | `validated` | `state/types.rs`, `state/hash.rs`, `state/stack.rs:269`, `state/stubs.rs`, `state/builder.rs:641`, `rules/resolution.rs:309+ModularTrigger`, `rules/abilities.rs` | Arcbound Worker | `stack/092` | — | CR 702.43a fully enforced; ETB counter placement (static) + dies trigger (targeted, dynamic counter count from pre_death_counters); StackObjectKind::ModularTrigger; auto-targets first artifact creature; 9 unit tests in modular.rs; script pending_review |
| Graft | 702.58 | P4 | `none` | — | — | — | — | ETB with +1/+1 counters; move to entering creatures |
| Evolve | 702.100 | P3 | `validated` | `state/types.rs:496`, `state/hash.rs:429`, `state/hash.rs:1244`, `state/stubs.rs:157`, `rules/abilities.rs:900`, `rules/resolution.rs:1015` | Cloudfin Raptor | `stack/093` | — | CR 702.100a fully enforced; intervening-if re-checked at resolution (CR 603.4); ETB creature comparison (P > P and/or T > T); StackObjectKind::EvolveTrigger; uses last-known P/T if entering creature leaves battlefield (ruling 2013-04-15); 10 unit tests in evolve.rs; script pending_review |
| Scavenge | 702.97 | P4 | `none` | — | — | — | — | Exile from graveyard → put +1/+1 counters on creature |
| Outlast | 702.107 | P4 | `none` | — | — | — | — | Tap + mana → +1/+1 counter (sorcery speed) |
| Amplify | 702.38 | P4 | `none` | — | — | — | — | Reveal creature cards from hand for +1/+1 counters |
| Adapt | 701.46 | P3 | `validated` | `state/types.rs:560`, `cards/card_definition.rs:793`, `effects/mod.rs:2731`, `state/hash.rs:445,2431` | Sharktocrab | `baseline/105` | — | CR 701.46a fully enforced; keyword action (not keyword ability); KeywordAbility::Adapt(u32) enum + Condition::SourceHasNoCountersOfType + Conditional effect; resolution-time check per ruling 2019-01-25 (activation always legal); re-adapt after losing counters verified; 6 unit tests in `adapt.rs`; game script approved |
| Bolster | 701.39 | P3 | `validated` | `cards/card_definition.rs:388`, `effects/mod.rs:971` | Cached Defenses | `baseline/104` | — | CR 701.39a fully enforced; keyword action (not keyword ability); chooses creature with least layer-aware toughness (ruling 2014-11-24); does NOT target (protection irrelevant); deterministic tie-break by smallest ObjectId; no-op if controller has no creatures; 8 unit tests in `bolster.rs`; game script approved |

---

## Section 10: Upkeep, Time & Phasing

Keywords involving time-based effects, phasing, and recurring costs.

| Ability | CR | Priority | Status | Engine File(s) | Card Def | Script | Depends On | Notes |
|---------|----|----------|--------|----------------|----------|--------|------------|-------|
| Cycling | 702.29 | P2 | `validated` | `state/types.rs:195`, `cards/card_definition.rs:145`, `state/hash.rs:316+1630+2220`, `rules/command.rs:182`, `rules/engine.rs:185`, `rules/abilities.rs:365`, `rules/events.rs:386` | Lonely Sandbar | `stack/061` | — | KeywordAbility::Cycling enum + AbilityDefinition::Cycling { cost }; Command::CycleCard dispatch; handle_cycle_card validates zone/keyword/mana, discards as cost, pushes draw onto stack; GameEvent::CardCycled emitted; 12 unit tests in `tests/cycling.rs`; game script approved |
| Suspend | 702.62 | P3 | `validated` | `state/types.rs:543`, `cards/card_definition.rs:252`, `rules/suspend.rs` (197 lines), `rules/command.rs:356`, `rules/events.rs:784`, `rules/engine.rs:304`, `rules/turn_actions.rs:35-91`, `rules/abilities.rs`, `rules/resolution.rs:1278-1449`, `state/hash.rs:439+2086` | Rift Bolt (#111) | `stack/102` | — | KeywordAbility::Suspend enum + AbilityDefinition::Suspend { cost, time_counters }; Command::SuspendCard special action (CR 116.2f); handle_suspend_card validates zone/keyword/timing/mana, exiles face-up with N time counters, is_suspended=true; GameEvent::CardSuspended (hash 86); upkeep trigger dispatch in turn_actions.rs queues SuspendCounterTrigger; resolution.rs removes counter, queues SuspendCastTrigger when last removed; cast trigger casts without paying mana cost (CR 702.62d), creatures gain haste (CR 702.62a); multiplayer: only owner's upkeep ticks; 9 unit tests in `tests/suspend.rs`; game script approved |
| Phasing | 702.26 | P4 | `none` | — | — | — | — | Phases out/in on untap step; deferred (corner case audit) |
| Cumulative Upkeep | 702.24 | P4 | `none` | — | — | — | — | Increasing cost each upkeep |
| Echo | 702.31 | P4 | `none` | — | — | — | — | Pay mana cost again on next upkeep or sacrifice |
| Fading | 702.32 | P4 | `none` | — | — | — | — | ETB with fade counters; remove each upkeep; sacrifice at 0 |
| Vanishing | 702.63 | P4 | `none` | — | — | — | — | ETB with time counters; remove each upkeep; sacrifice at 0 |
| Forecast | 702.57 | P4 | `none` | — | — | — | — | Reveal from hand during upkeep for effect |
| Recover | 702.59 | P4 | `none` | — | — | — | — | When a creature dies, return this from graveyard |
| Dredge | 702.52 | P2 | `validated` | `state/types.rs:205`, `state/hash.rs:318+1646+1656`, `rules/command.rs:187`, `rules/events.rs:672-692`, `rules/replacement.rs:412-1492`, `rules/engine.rs:200`, `rules/turn_actions.rs:108`, `effects/mod.rs:1532` | Golgari Grave-Troll | `replacement/014` | — | KeywordAbility::Dredge(u32) enum; GameEvent::DredgeChoiceRequired + Dredged; Command::ChooseDredge; DrawAction::DredgeAvailable; check_would_draw_replacement dredge scan; handle_choose_dredge + draw_card_skipping_dredge; 13 unit tests in `tests/dredge.rs`; choose_dredge harness action; game script approved (9/9 assertions pass) |

---

## Section 11: Commander & Multiplayer

Keywords specifically relevant to Commander or multiplayer.

| Ability | CR | Priority | Status | Engine File(s) | Card Def | Script | Depends On | Notes |
|---------|----|----------|--------|----------------|----------|--------|------------|-------|
| Partner | 702.124 | P1 | `validated` | `rules/commander.rs` | — | `commander/` scripts | — | Two commanders; deck validation enforced |
| Companion | 702.139 | P1 | `validated` | `rules/commander.rs`, `rules/engine.rs` | — | `commander/` scripts | — | Start in sideboard; bring to hand for {3} |
| Partner With | 702.124 | P3 | `validated` | `state/types.rs:213`, `state/stack.rs:394`, `rules/abilities.rs:1001,2104`, `rules/resolution.rs:1572`, `rules/commander.rs:487-540` | Pir, Imaginative Rascal; Toothy, Imaginary Friend | `baseline/107` | Partner | 10 unit tests in `partner_with.rs`; ETB trigger searches library; deck validation cross-checks names (CR 702.124j); prevents mixing with plain Partner (CR 702.124f) |
| Friends Forever | 702.124 | P4 | `none` | — | — | — | Partner | Partner variant from Stranger Things Secret Lair |
| Choose a Background | 702.124 | P4 | `none` | — | — | — | Partner | Partner variant for Background enchantments |
| Doctor's Companion | 702.124 | P4 | `none` | — | — | — | Partner | Partner variant from Doctor Who |

---

## Section 12: Set-specific & Niche

Keywords from specific sets, used on few cards. Implement when a card definition needs them.

| Ability | CR | Priority | Status | Engine File(s) | Card Def | Script | Depends On | Notes |
|---------|----|----------|--------|----------------|----------|--------|------------|-------|
| Morph | 702.37 | P3 | `none` | — | — | — | — | Cast face-down as 2/2 for {3}; turn face-up for morph cost. Deferred (corner case audit) |
| Disguise | 702.162 | P4 | `none` | — | — | — | Morph | Morph variant with ward {2} |
| Megamorph | 702.37 | P4 | `none` | — | — | — | Morph | Morph that adds +1/+1 counter when turned up |
| Manifest | 701.34 | P4 | `none` | — | — | — | Morph | Put top card face-down as 2/2; turn up if creature |
| Cloak | 701.56 | P4 | `none` | — | — | — | Manifest | Manifest variant with ward {2} |
| Mutate | 702.140 | P3 | `none` | — | — | — | — | Merge with creature; deferred (corner case audit) |
| Changeling | 702.73 | P2 | `validated` | `state/types.rs:286-293` (KeywordAbility::Changeling + ALL_CREATURE_TYPES:296-376), `state/hash.rs:360-361`, `state/continuous_effect.rs:139-145` (AddAllCreatureTypes), `rules/layers.rs:61-76` (inline CDA check + apply arm:326-334), `tools/replay-viewer/src/view_model.rs:602` | Universal Automaton | `layers/072` | — | CR 702.73a CDA: "This object is every creature type." Applied in Layer 4 before non-CDA effects (CR 613.3); functions in all zones (CR 604.3); ALL_CREATURE_TYPES lazy static (~290+ subtypes from CR 205.3m); LayerModification::AddAllCreatureTypes for Maskwood Nexus-style effects; 7 unit tests in `tests/changeling.rs`; game script pending_review |
| Crew | 702.122 | P2 | `validated` | `state/types.rs:302`, `rules/command.rs:245`, `rules/engine.rs:234`, `rules/abilities.rs:1246`, `testing/replay_harness.rs:408` | Smuggler's Copter | `combat/075` | 15 tests in `crew.rs`; script `pending_review` (multi-turn attack gap, same as 069/070) |
| Saddle | 702.163 | P4 | `none` | — | — | — | Crew | Crew variant for Mounts |
| Prototype | 702.157 | P4 | `none` | — | — | — | — | Alternative smaller casting |
| Living Metal | — | P4 | `none` | — | — | — | — | Artifact is also a creature on your turn |
| Totem Armor | 702.89 | P4 | `none` | — | — | — | Enchant | Aura destroyed instead of enchanted permanent |
| Soulbond | 702.95 | P4 | `none` | — | — | — | — | Pair with another creature for shared abilities |
| Haunt | 702.55 | P4 | `none` | — | — | — | — | When this dies, exile haunting a creature |
| Extort | 702.101 | P3 | `validated` | `state/types.rs:324`, `state/game_object.rs:174`, `cards/card_definition.rs:224`, `effects/mod.rs:261`, `rules/abilities.rs:640`, `state/builder.rs:561`, `state/hash.rs:376`, `tools/replay-viewer/src/view_model.rs:610` | Syndic of Tithes | `stack/078` | — | CR 702.101a+b: triggered ability on spell cast, may pay {W/B}, drain 1 from each opponent; multiple instances trigger separately; 7 unit tests in `tests/extort.rs` |
| Cipher | 702.99 | P4 | `none` | — | — | — | — | Encode spell on creature; cast copy on combat damage |
| Bloodthirst | 702.54 | P4 | `none` | — | — | — | — | ETB with +1/+1 counters if opponent was dealt damage |
| Bloodrush | — | P4 | `none` | — | — | — | — | Ability word; discard to pump attacking creature |
| Devoid | 702.114 | P4 | `validated` | state/types.rs:609-615, state/hash.rs:463-464, rules/layers.rs:74-83 | Forerunner of Slaughter | baseline/111 | CR 702.114a fully enforced; CDA in Layer 5 (ColorChange) clears colors; functions in all zones (CR 604.3); 8 unit tests in devoid.rs; script approved | Colorless regardless of mana cost |
| Ingest | 702.115 | P4 | `validated` | state/types.rs:629-638, state/stubs.rs:230-242, state/stack.rs:407-427, state/hash.rs:467-468+1090-1092+1373-1374, rules/abilities.rs:1757-1840+2218-2226, rules/resolution.rs:1634-1671 | Mist Intruder | baseline/113 | CR 702.115a+b fully enforced; triggered ability on combat damage to player; multiple instances trigger separately (702.115b); empty library is safe no-op; face-up exile (default); 6 unit tests in ingest.rs; TUI stack_view.rs:80-81 + view_model.rs:473-474,687 | Combat damage to player → exile top card of library |
| Wither | 702.80 | P3 | `validated` | state/types.rs:481, state/hash.rs:422, rules/combat.rs:863-1006, effects/mod.rs:206-241 | Boggart Ram-Gang | combat/091 | CR 702.80a fully enforced; combat + non-combat damage to creatures places -1/-1 counters instead of marking damage; 6 unit tests in keywords.rs; script pending_review | Damage dealt as -1/-1 counters |
| Infect | 702.90 | P3 | `validated` | state/types.rs:511-520, state/hash.rs:433-444, rules/events.rs:357-374, rules/combat.rs:863-1073, effects/mod.rs:143-275 | Glistener Elf | combat/092 | CR 702.90 fully enforced; creature damage as -1/-1 counters (reusing Wither path); player damage as poison counters; 9 unit tests in tests/keywords.rs; poison SBA (704.5c) was pre-existing |
| Poisonous | 702.70 | P4 | `none` | — | — | — | — | Combat damage to player → poison counters |
| Toxic | 702.156 | P4 | `none` | — | — | — | — | Combat damage to player → poison counters (fixed number) |
| Corrupted | — | P4 | `none` | — | — | — | — | Ability word; if opponent has 3+ poison counters |
| Hideaway | 702.75 | P3 | `validated` | types.rs:544, stack.rs:370, stubs.rs:205, game_object.rs:409, abilities.rs:921, abilities.rs:2011, resolution.rs:1468, events.rs:843, effects/mod.rs:1593, lands.rs:136, engine.rs:72, hash.rs:439 | Windbrisk Heights (#112) | baseline/103 | 7 unit tests in hideaway.rs; ETB trigger, resolution, exile tracking, empty-library edge, face-down, PlayExiledCard, negative test |
| Retrain | — | P4 | `n/a` | — | — | — | — | Digital-only (MTG Arena) |
| Perpetually | — | P4 | `n/a` | — | — | — | — | Digital-only (MTG Arena) |
| Conjure | — | P4 | `n/a` | — | — | — | — | Digital-only (MTG Arena) |
| Seek | — | P4 | `n/a` | — | — | — | — | Digital-only (MTG Arena) |
| Specialize | — | P4 | `n/a` | — | — | — | — | Digital-only (Alchemy) |
| Intensity | — | P4 | `n/a` | — | — | — | — | Digital-only (Alchemy) |
| Spellbook | — | P4 | `n/a` | — | — | — | — | Digital-only (Alchemy) |
| Draft | — | P4 | `n/a` | — | — | — | — | Digital-only (Alchemy) |
| Boon | — | P4 | `n/a` | — | — | — | — | Digital-only (Alchemy) |
| Craft | 702.158 | P4 | `none` | — | — | — | — | Exile from battlefield + exile materials → transform |
| Connive | 702.153 | P3 | `validated` | effects/mod.rs:L1492; events.rs:L760; game_object.rs:L190; card_definition.rs:L699 | Raffine's Informant | stack/095 | 7 tests in connive.rs | CR 701.50a/e: Draw N, discard N, +1/+1 counter for each nonland discarded |
| Casualty | 702.154 | P4 | `none` | — | — | — | — | Sacrifice creature with power >= N → copy spell |
| Alliance | — | P4 | `none` | — | — | — | — | Ability word; trigger when creature ETBs under your control |
| Ravenous | — | P4 | `none` | — | — | — | — | ETB with X +1/+1 counters; draw if X >= 5 |
| Squad | 702.159 | P4 | `none` | — | — | — | — | Pay squad cost N times → N token copies on ETB |
| Enrage | — | P4 | `none` | — | — | — | — | Ability word; trigger when dealt damage |
| Ascend | 702.131 | P3 | `validated` | `state/types.rs` (KeywordAbility enum), `rules/sba.rs:91-176` (check_ascend SBA function), `rules/resolution.rs:192-207` (instant/sorcery ascend at resolution), `rules/events.rs:762` (GameEvent::CitysBlessingGained), `state/game_object.rs` (has_citys_blessing on PlayerState) | Wayward Swordtooth | `baseline/096` | 7 tests in ascend.rs | CR 702.131a/b/c: Static Ascend when 10+ permanents with keyword source; spell Ascend at resolution; blessing permanent once gained; city's blessing is irremovable designation |
| Treasure tokens | 111.10a | P2 | `validated` | `state/game_object.rs:42-81` (ManaAbility with sacrifice_self + any_color), `rules/mana.rs:95-149` (sacrifice cost + any-color handling), `cards/card_definition.rs:629-683` (TokenSpec.mana_abilities, treasure_token_spec helper), `effects/mod.rs:1700-1714` (make_token populates mana_abilities), `state/hash.rs:457-462,1816-1830` | Strike It Rich | `stack/073` | — | CR 111.10a fully enforced; colorless Treasure artifact token with "{T}, Sacrifice this token: Add one mana of any color"; mana ability resolves without stack (CR 605.3b); sacrifice as cost (CR 602.2c); token ceases to exist in graveyard (CR 111.7/704.5d); 9 unit tests in `tests/treasure_tokens.rs`; game script pending_review (all assertions pass) |
| Food tokens | CR 111.10b | P3 | `validated` | card_definition.rs:L815–845; effects/mod.rs (make_token propagation) | Bake into a Pie | stack/097 (pending_review) | 11 unit tests in food_tokens.rs | CR 111.10b: colorless Food artifact token with {2}, {T}, Sacrifice → gain 3 life. Validated: food_token_spec() helper, TokenSpec.activated_abilities, make_token() propagates abilities, unit tests cover activation/cost/sacrifice/SBA, game script exercises spell→token→activate→resolve. |
| Clue tokens | CR 111.10f | P3 | `validated` | `cards/card_definition.rs:847-882` (clue_token_spec), `cards/mod.rs:16`, `lib.rs:9` | Thraben Inspector | `stack/098` (pending_review) | 11 unit tests in clue_tokens.rs | CR 111.10f: colorless Clue artifact token with {2}, Sacrifice → draw a card; no tap cost (unlike Food); clue_token_spec() helper, TokenSpec.activated_abilities, make_token() propagates abilities |
| Blood tokens | — | P4 | `none` | — | — | — | — | Predefined token: {1}, tap, discard, sacrifice → draw |
| Prowess | 702.108 | P1 | `validated` | `state/types.rs`, `state/hash.rs`, `state/game_object.rs`, `state/builder.rs`, `state/continuous_effect.rs`, `rules/abilities.rs`, `effects/mod.rs` | Monastery Swiftspear | `stack/056` | — | KeywordAbility::Prowess enum + TriggerEvent::ControllerCastsNoncreatureSpell dispatch; EffectFilter::Source in continuous_effect.rs; TriggeredAbilityDef auto-expansion in builder.rs; 8 unit tests in `tests/prowess.rs`; game script 056 (pending_review, all assertions pass) |
| Regenerate | 701.19 | P3 | `validated` | `cards/card_definition.rs:484` (Effect::Regenerate), `state/replacement_effect.rs` (WouldBeDestroyed, Regenerate mod), `rules/replacement.rs:1609` (check_regeneration_shield, apply_regeneration), `effects/mod.rs:534+1457`, `rules/sba.rs:336`, `rules/events.rs:799` | Drudge Skeletons | `stack/100` | 10 tests in regenerate.rs | CR 701.19a: next time destroyed, remove damage, tap, remove from combat; one-shot replacement shield; zero-toughness (704.5f) bypasses regeneration; indestructible prevents before regeneration check |
| Proliferate | 701.34 | P3 | `validated` | `effects/mod.rs:1518`, `rules/events.rs:833`, `state/game_object.rs:198`, `rules/abilities.rs:1542` | Inexorable Tide | `stack/101` | 14 tests in `proliferate.rs` | Effect::Proliferate + GameEvent::Proliferated + TriggerEvent::ControllerProliferates; auto-selects all eligible permanents/players (interactive choice deferred M10+); game script `pending_review` |
| Transform | 701.28 | P3 | `none` | — | — | — | — | Keyword action: flip DFC to other face |
| Daybound/Nightbound | 702.145 | P4 | `none` | — | — | — | Transform | DFC auto-transform based on day/night cycle |
| Investigate | 701.16 | P3 | `validated` | `cards/card_definition.rs:325` (Effect::Investigate), `effects/mod.rs:484` (execution), `rules/events.rs:706` (GameEvent::Investigated), `state/game_object.rs:193` (TriggerEvent::ControllerInvestigates), `state/hash.rs` (hash arms), `rules/abilities.rs:1447` (trigger dispatch) | Magnifying Glass, Thraben Inspector | `stack/099` (pending_review) | 6 unit tests in investigate.rs | CR 701.16a: "Investigate" means "Create a Clue token"; Effect::Investigate wraps clue_token_spec(1) x N; TriggerEvent::ControllerInvestigates wired; Thraben Inspector uses Effect::Investigate on ETB |
| Amass | 701.44 | P4 | `none` | — | — | — | — | Put +1/+1 counters on Army token or create one |
| Discover | 702.161 | P4 | `none` | — | — | — | Cascade | Cascade variant without the free cast restriction |
| Forage | 701.55 | P4 | `none` | — | — | — | — | Sacrifice a Food or exile 3 cards from graveyard |
| Offspring | 702.167 | P4 | `none` | — | — | — | — | Pay offspring cost → create 1/1 token copy on ETB |
| Impending | 702.168 | P4 | `none` | — | — | — | — | Cast for less as non-creature with time counters |
| Gift | 702.169 | P4 | `none` | — | — | — | — | Choose an opponent to receive a gift |
| Collect evidence | 701.53 | P4 | `none` | — | — | — | — | Exile cards from graveyard with total MV >= N |
| Suspect | 701.52 | P4 | `none` | — | — | — | — | Menace + can't block |
| Surveil | 701.25 | P2 | `validated` | `cards/card_definition.rs:293-303` (Effect::Surveil), `rules/events.rs:678-684` (GameEvent::Surveilled), `state/game_object.rs:147-150` (TriggerEvent::ControllerSurveils), `cards/card_definition.rs:577-582` (TriggerCondition::WheneverYouSurveil), `effects/mod.rs:917-953` (execution), `rules/abilities.rs:846-865` (trigger dispatch), `testing/replay_harness.rs:635-651` (enrichment), `state/hash.rs:959,1736,1996,2259` (hash arms) | Consider | `stack/071` | — | CR 701.25a/c/d fully enforced; deterministic fallback (all surveilled cards go to graveyard); surveil 0 suppresses event (CR 701.25c); event fires even with empty/partial library (CR 701.25d); WheneverYouSurveil trigger pipeline complete; 7 unit tests in `tests/surveil.rs`; game script pending_review (all assertions pass) |
| Adapt (keyword action) | 701.46 | P3 | `validated` | — | — | — | — | See Section 9 (primary row); implementation tracked there |
| Venture/Dungeon | 309 | P4 | `n/a` | — | — | — | — | Dungeon cards not in Commander precons; very niche |
| The Ring Tempts You | — | P4 | `n/a` | — | — | — | — | LotR-specific mechanic |

---

## Section 13: Non-keyword Ability Patterns

Common ability patterns that appear across many cards but aren't formal CR 702 keywords.

| Pattern | Priority | Status | Engine File(s) | Card Def | Script | Depends On | Notes |
|---------|----------|--------|----------------|----------|--------|------------|-------|
| Tap-for-mana | P1 | `validated` | `cards/definitions.rs`, `effects/` | 14 land/artifact cards | `baseline/` scripts | — | Mana ability activation fully working |
| Sacrifice-to-draw | P1 | `validated` | `cards/definitions.rs`, `effects/` | Commander's Sphere, Mind Stone, Arcane Signet | Scripts 053, 054 | — | Activated ability with sacrifice cost |
| ETB trigger | P1 | `complete` | `cards/card_definition.rs:466`, `rules/resolution.rs` | Wall of Omens, Solemn Simulacrum | — | — | `WhenEntersBattlefield` TriggerCondition; runtime dispatch works |
| Dies trigger | P1 | `validated` | `cards/card_definition.rs:468`, `rules/abilities.rs:428-463`, `testing/replay_harness.rs:406-424` | Solemn Simulacrum | Scripts 019, 058 | — | `TriggerEvent::SelfDies` dispatched in `check_triggers`; `enrich_spec_from_def` converts `WhenDies`; 10 unit tests; 2 approved scripts |
| Attack trigger | P1 | `validated` | `state/game_object.rs:111` (TriggerEvent::SelfAttacks), `cards/card_definition.rs:470` (TriggerCondition::WhenAttacks), `testing/replay_harness.rs:426-447` (enrichment), `rules/abilities.rs:377-388` (dispatch — CR 508.1m, 508.3a), `rules/combat.rs:290-298` (flush) | Audacious Thief | `combat/011` | — | Full pipeline: enum + enrichment + dispatch + flush + card def + script; 5 unit tests in `tests/abilities.rs:1790-2253` + 1 in `tests/combat.rs:744`; script `pending_review` (10/10 assertions pass) |
| Combat damage trigger | P1 | `validated` | `state/game_object.rs:135`, `state/hash.rs:901`, `rules/abilities.rs:561`, `testing/replay_harness.rs:494` | Scroll Thief | `combat/013` | — | TriggerEvent::SelfDealsCombatDamageToPlayer enum + dispatch via CombatDamageDealt match; enrichment in replay_harness; 4 unit tests in `tests/combat.rs:1539`; game script approved |
| Opponent-casts trigger | P1 | `validated` | `state/game_object.rs:140`, `state/stubs.rs:59`, `state/hash.rs:905,1852`, `rules/abilities.rs:456,725`, `testing/replay_harness.rs:509-515`, `cards/card_definition.rs:492` | Rhystic Study | `stack/059` | — | TriggerEvent::OpponentCastsSpell enum + dispatch in SpellCast arm; triggering_player on PendingTrigger; WheneverOpponentCastsSpell enrichment in replay_harness; 5 unit tests in `tests/effects.rs`; game script approved (12/12 assertions) |
| Search library | P1 | `complete` | `effects/` | Wayfarer's Bauble, Evolving Wilds, Terramorphic Expanse, Cultivate | — | — | `SearchLibrary` effect works; harness doesn't emit player command yet |
| Destroy + compensate | P1 | `validated` | `effects/` | Beast Within, Generous Gift, Pongify, Rapid Hybridization | `baseline/` scripts | — | Destroy target + create token for controller |
| Mass removal | P1 | `validated` | `effects/` | Wrath of God, Damnation, Supreme Verdict, Blasphemous Act | `baseline/` scripts | — | Destroy/damage all creatures |
| Counter spell | P1 | `validated` | `effects/` | Counterspell, Negate, Swan Song, Arcane Denial | `stack/` scripts | — | Counter target spell on stack |
| Global replacement | P1 | `validated` | `effects/`, `rules/replacement.rs` | Rest in Peace, Leyline of the Void | `replacement/` scripts | — | Replace zone change events globally |
| Equipment keyword grant | P1 | `validated` | `cards/definitions.rs`, `rules/layers.rs` | Lightning Greaves, Swiftfoot Boots | `layers/` scripts | — | Layer 6 continuous effect granting keywords |
| Modal choice | P2 | `none` | — | — | — | — | "Choose one —" modal spells not yet supported |
| Declare attackers action | P1 | `validated` | `rules/combat.rs:29-310`, `testing/replay_harness.rs:279-310`, `testing/script_schema.rs:342-357` | Llanowar Elves | `combat/015`, `combat/016` | — | AttackerDeclaration struct; translate_player_action "declare_attackers" arm resolves creature names to ObjectIds + player names to AttackTarget; deterministic default target (alphabetically sorted opponents); find_on_battlefield_by_name helper; 6 unit tests in `tests/combat_harness.rs`; 2 game scripts (12+21 assertions) |
| Declare blockers action | P1 | `validated` | `rules/combat.rs:312-590`, `testing/replay_harness.rs:315-326`, `testing/script_schema.rs:359-369` | Elvish Mystic | `combat/015`, `combat/016` | — | BlockerDeclaration struct; translate_player_action "declare_blockers" arm resolves blocker+attacker names to ObjectIds via find_on_battlefield_by_name; 6 unit tests in `tests/combat_harness.rs`; 2 game scripts (12+21 assertions) |

---

## Ability Addition Workflow

End-to-end process for taking an ability from `none` → `validated`. Use `/next-ability`
to pick one, then follow these steps. Not every ability needs every step — simple keywords
(e.g., evasion) skip step 3; non-keyword patterns may skip step 1.

### Step 1: Add or verify the enum variant

**File**: `crates/engine/src/state/types.rs` (`KeywordAbility` enum)

- Add a new variant if one doesn't exist (e.g., `Cycling`).
- Add a doc comment citing the CR rule number.
- Update `crates/engine/src/state/hash.rs` — add a `HashInto` arm for the new variant
  with the next available discriminant.
- If the ability takes parameters (like `ProtectionFrom(ProtectionQuality)`), model them
  as enum data.

**Skip if**: The ability is a non-keyword pattern (Section 13), or the variant already
exists (check the `partial` row's notes).

### Step 2: Implement the rule enforcement

**Where it goes depends on the ability type**:

| Ability Type | File | Example |
|-------------|------|---------|
| Evasion / blocking restriction | `rules/combat.rs` | Flying, Menace, Intimidate, Fear |
| Targeting restriction | `rules/mod.rs`, `rules/protection.rs` | Hexproof, Shroud, Ward |
| Damage modification | `rules/combat.rs` | Deathtouch, Lifelink, Trample |
| SBA-related | `rules/sba.rs` | Indestructible |
| Casting permission / timing | `rules/casting.rs` | Flash, Split Second |
| Casting cost modification | `rules/casting.rs` | Convoke, Delve |
| Spell copy / trigger on cast | `rules/casting.rs`, `rules/copy.rs` | Storm, Cascade |
| Alternative casting zone | `rules/casting.rs` | Flashback, Madness |
| ETB / dies / attack trigger | `rules/triggers.rs` or `rules/resolution.rs` | Persist, Undying |
| Commander-specific | `rules/commander.rs` | Partner, Companion |
| Continuous effect | `rules/layers.rs` | Equipment keyword grant |
| New effect primitive | `effects/` | SearchLibrary, CreateToken |

**Checklist**:
- Cite the CR rule number in a comment at the enforcement site.
- Handle multiplayer correctly (N players, not just 2).
- Check if the ability interacts with replacement effects, SBAs, or the layer system.
- Read `memory/gotchas-rules.md` for the subsystem you're touching.

### Step 3: Wire triggers (if applicable)

For abilities that trigger on game events (ETB, dies, attacks, casts, etc.):

1. Add or reuse a `TriggerCondition` variant in `cards/card_definition.rs`.
2. Wire the trigger check in the appropriate event handler (e.g., `check_triggers` in
   `rules/triggers.rs`, or inline in `rules/combat.rs` for combat triggers).
3. Ensure the trigger creates a proper stack object with the correct controller.

### Step 4: Write unit tests

**File**: `crates/engine/tests/` (pick the file matching the subsystem, or `abilities.rs`
for keyword abilities).

- Cite the CR rule in the test name and doc comment.
- Test the positive case (ability works as expected).
- Test at least one negative case (ability doesn't fire when it shouldn't).
- For combat keywords: test with first strike, double strike, and multiplayer scenarios.
- Run: `~/.cargo/bin/cargo test --all`

### Step 5: Add a card definition

Use the `card-definition-author` agent or add manually to
`crates/engine/src/cards/definitions.rs`.

- Pick a real Commander-playable card that uses the ability.
- Follow the existing card definition patterns (see card #45 Lightning Greaves for
  equipment, card #1 Sol Ring for mana abilities).
- Register it in the `CardRegistry`.

### Step 6: Write a game script

Use the `game-script-generator` agent or write manually to
`test-data/generated-scripts/<category>/`.

- Script should exercise the ability end-to-end in a realistic game scenario.
- Include assertions checking that the ability produced the expected state change.
- Add the CR rule number to `cr_sections_tested` in script metadata.
- Run via the replay harness to verify.

### Step 7: Update coverage doc

Update the ability's row in this document:
- Set **Status** to `validated` (or `complete` if no script yet).
- Fill in **Engine File(s)** with the implementation location.
- Fill in **Card Def** with the card name.
- Fill in **Script** with the script ID.
- Update the **Summary** table counts.

Or run `/audit-abilities <ability-name>` to refresh automatically.

---

## Priority Gaps

Top unresolved gaps ordered by priority.

### P1 Gaps (on existing cards or blocking scripts)

All P1 gaps resolved. 40/42 validated, 2 complete (ETB trigger, Search library).

### P2 Gaps (Commander staples)

1. **Modal choice** — "Choose one" modal spells; non-keyword pattern, no implementation.

**Resolved**: Crew (CR 702.122) — validated 2026-02-27 (script combat/075, Smuggler's Copter, 15 unit tests in crew.rs).

**Resolved**: Declare attackers/blockers harness action — validated 2026-02-26 (scripts combat/015+016, 6 unit tests in combat_harness.rs). Flashback (CR 702.34) — validated 2026-02-26 (script 060, Think Twice + Faithless Looting). Cycling (CR 702.29) — validated 2026-02-26 (script 061, Lonely Sandbar). Dredge (CR 702.52) — validated 2026-02-26 (script replacement/014, Golgari Grave-Troll). Convoke (CR 702.51) — validated 2026-02-26 (script stack/063, Siege Wurm, 12 unit tests in convoke.rs). Delve (CR 702.66) — validated 2026-02-26 (script stack/064, Treasure Cruise, 10 unit tests in delve.rs). Kicker (CR 702.33) — validated 2026-02-26 (script stack/065, Burst Lightning + Torch Slinger, 10 unit tests in kicker.rs). Split Second (CR 702.61) — validated 2026-02-26 (script stack/066, Krosan Grip, 8 unit tests in split_second.rs). Exalted (CR 702.83) — validated 2026-02-26 (script combat/067, Akrasan Squire, 8 unit tests in exalted.rs). Annihilator (CR 702.86) — validated 2026-02-26 (script combat/068, Ulamog's Crusher, 8 unit tests in annihilator.rs). Persist (CR 702.79) — validated 2026-02-26 (script combat/069, Kitchen Finks, 6 unit tests in persist.rs). Undying (CR 702.93) — validated 2026-02-26 (script combat/070, Young Wolf, 6 unit tests in undying.rs). Surveil (CR 701.25) — validated 2026-02-26 (script stack/071, Consider, 7 unit tests in surveil.rs). Changeling (CR 702.73) — validated 2026-02-26 (script layers/072, Universal Automaton, 7 unit tests in changeling.rs). Treasure tokens (CR 111.10a) — validated 2026-02-26 (script stack/073, Strike It Rich, 9 unit tests in treasure_tokens.rs). Evoke (CR 702.74) — validated 2026-02-27 (script stack/074, Mulldrifter, 8 unit tests in evoke.rs).

### P3 Gaps (Commander-relevant, less common)

**Resolved**: Battle Cry (CR 702.91) — validated 2026-02-27 (script combat/076, Signal Pest, 7 unit tests in battle_cry.rs).

**Resolved**: Extort (CR 702.101) — validated 2026-02-27 (script stack/078, Syndic of Tithes, 7 unit tests in extort.rs).

**Resolved**: Improvise (CR 702.126) — validated 2026-02-27 (script stack/079, Reverse Engineer, 12 unit tests in improvise.rs).

**Resolved**: Bestow (CR 702.103) — validated 2026-02-27 (script layers/081, Boon Satyr, 9 unit tests in bestow.rs).

**Resolved**: Living Weapon (CR 702.92) — validated 2026-02-27 (script stack/082, Batterskull, 6 unit tests in living_weapon.rs).

**Resolved**: Madness (CR 702.35) — validated 2026-02-27 (script stack/083, Fiery Temper, 11 unit tests in madness.rs).

**Resolved**: Miracle (CR 702.94) — validated 2026-02-27 (script stack/084, Terminus, 11 unit tests in miracle.rs).

**Resolved**: Escape (CR 702.138) — validated 2026-02-27 (script stack/085, Ox of Agonas, 16 unit tests in escape.rs).

**Resolved**: Foretell (CR 702.143) — validated 2026-02-27 (script stack/086, Saw It Coming, 18 unit tests in foretell.rs).

**Resolved**: Unearth (CR 702.84) — validated 2026-02-27 (script stack/087, Dregscape Zombie, 12 unit tests in unearth.rs).

**Resolved**: Fear (CR 702.36) — validated 2026-02-27 (script combat/080, Severed Legion, 7 unit tests in keywords.rs:1818).

**Resolved**: Wither (CR 702.80) — validated 2026-02-28 (script combat/091, Boggart Ram-Gang, 6 unit tests in keywords.rs).

**Resolved**: Modular (CR 702.43) — validated 2026-02-28 (script stack/092, Arcbound Worker, 9 unit tests in modular.rs).

**Resolved**: Clue tokens (CR 111.10f) — validated 2026-02-28 (script stack/098, Thraben Inspector, 11 unit tests in clue_tokens.rs).

**Resolved**: Regenerate (CR 701.19) — validated 2026-02-28 (script stack/100, Drudge Skeletons, 10 unit tests in regenerate.rs).

**Resolved**: Proliferate (CR 701.34) — validated 2026-02-28 (script stack/101, Inexorable Tide, 14 unit tests in proliferate.rs).

**Resolved**: Myriad (CR 702.116) — validated 2026-02-28 (script combat/093, Warchief Giant, 7 unit tests in myriad.rs).

**Resolved**: Suspend (CR 702.62) — validated 2026-02-28 (script stack/102, Rift Bolt, 9 unit tests in suspend.rs).

**Resolved**: Hideaway (CR 702.75) — validated 2026-02-28 (script baseline/103, Windbrisk Heights, 7 unit tests in hideaway.rs).

**Resolved**: Bolster (CR 701.39) — validated 2026-02-28 (script baseline/104, Cached Defenses, 8 unit tests in bolster.rs).

**Resolved**: Adapt (CR 701.46) — validated 2026-02-28 (script baseline/105, Sharktocrab, 6 unit tests in adapt.rs).

**Resolved**: Overload (CR 702.96) — validated 2026-02-28 (script baseline/108, Vandalblast, 11 unit tests in overload.rs).

### P4 Gaps (niche / historical)

**Resolved**: Shadow (CR 702.28) — validated 2026-02-28 (script combat/106, Dauthi Slayer, 7 unit tests in shadow.rs).

**Resolved**: Horsemanship (CR 702.31) — validated 2026-02-28 (script combat/109, Shu Cavalry, 7 unit tests in horsemanship.rs).

**Resolved**: Skulk (CR 702.118) — validated 2026-02-28 (script combat/110, Furtive Homunculus, 7 unit tests in skulk.rs).

**Resolved**: Devoid (CR 702.114) — validated 2026-02-28 (script baseline/111, Forerunner of Slaughter, 8 unit tests in devoid.rs).

**Resolved**: Decayed (CR 702.147) — validated 2026-02-28 (script baseline/112, Shambling Ghast, 8 unit tests in decayed.rs).

**Resolved**: Ingest (CR 702.115) — validated 2026-03-01 (script baseline/113, Mist Intruder, 6 unit tests in ingest.rs).

**Resolved**: Bushido (CR 702.45) — validated 2026-03-01 (script combat/115, Devoted Retainer, 7 unit tests in bushido.rs).

**Resolved**: Rampage (CR 702.23) — validated 2026-03-01 (script combat/116, Wolverine Pack, 8 unit tests in rampage.rs).

**Resolved**: Provoke (CR 702.39) — validated 2026-03-01 (script combat/117, Goblin Grappler, 7 unit tests in provoke.rs).

**Resolved**: Afflict (CR 702.130) — validated 2026-03-01 (script combat/118, Khenra Eternal, 6 unit tests in afflict.rs).

**Resolved**: Renown (CR 702.112) — validated 2026-03-01 (script combat/119, Topan Freeblade, 7 unit tests in renown.rs).

**Resolved**: Training (CR 702.149) — validated 2026-03-01 (script combat/120, Gryff Rider, 7 unit tests in training.rs).

**Resolved**: Melee (CR 702.121) — validated 2026-03-01 (script combat/121, Wings of the Guard, 7 unit tests in melee.rs).
