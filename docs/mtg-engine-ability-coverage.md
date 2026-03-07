# MTG Engine — Ability Coverage Audit

> Living document. Refresh with `/audit-abilities`.
> Last audited: 2026-03-07 (Soulbond 702.95 validated: paired_with on GameObject, SoulbondSelfETB+SoulbondOtherETB triggers, SOK 49, Silverblade Paladin, script stack/167, 10 tests)

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
| P3       | 40    | 37        | 0        | 0       | 3    | 0   |
| P4       | 105   | 61        | 0        | 0       | 32   | 12  |
| **Total**| **204**| **154**  | **2**    | **0**   | **36**| **12** |

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
| Retrace | 702.81 | P4 | `validated` | `state/types.rs:762-770`, `state/hash.rs:513-514`, `rules/casting.rs:69,186-221,434,711-765,993-997`, `rules/command.rs:167-176`, `rules/engine.rs:97,119`, `testing/replay_harness.rs:739-764`, `testing/script_schema.rs:269-270` | Flame Jab | `combat/126` | — | CR 702.81a: static ability — cast from graveyard by paying normal mana cost + discarding a land card as additional cost (CR 118.8, not alternative); KeywordAbility::Retrace disc 89; `retrace_discard_land: Option<ObjectId>` on CastSpell command; validation: card has Retrace keyword, is in graveyard, land is in hand + is land type; card returns to graveyard on resolution (not exile, unlike Flashback); sorcery-speed timing preserved; `cast_spell_retrace` harness action; 11 unit tests in `tests/retrace.rs`; game script approved |
| Jump-Start | 702.133 | P4 | `validated` | `state/types.rs:780-782` (KeywordAbility::JumpStart disc 90), `state/hash.rs:516`, `rules/casting.rs:70-71,117-118,205-225,305-314,802-826,1083-1089,1223-1224`, `rules/resolution.rs:83-93,539-544,2289-2291`, `rules/command.rs:178-190`, `state/stack.rs:141` | Radical Idea (`defs/radical_idea.rs`) | `combat/127` | — | CR 702.133a fully enforced; two static abilities: (1) cast from graveyard by paying normal mana cost + discarding any card from hand as additional cost (CR 601.2b,f-h), (2) exile instead of going anywhere else when leaving the stack (resolve, counter, fizzle); `cast_with_jump_start: bool` + `jump_start_discard: Option<ObjectId>` on CastSpell command; instant/sorcery type validation; discard-in-hand validation; mutual exclusion with flashback; sorcery-speed timing preserved; `cast_spell_jump_start` harness action; 12 unit tests in `tests/jump_start.rs`; game script pending_review |
| Aftermath | 702.127 | P4 | `validated` | `state/types.rs:783-793`, `cards/card_definition.rs:279-303`, `rules/casting.rs:231-270,646-778,984-986,1732-1765`, `rules/resolution.rs:161`, `rules/command.rs:197`, `state/hash.rs:517-518,1592,3087-3088`, `testing/replay_harness.rs:835` | Cut // Ribbons (defs/cut_ribbons.rs) | `combat/128` | — | CR 702.127a fully enforced: three static abilities (graveyard-only cast, graveyard zone restriction, exile-on-stack-departure); KeywordAbility::Aftermath disc 91 + AbilityDefinition::Aftermath disc 24 (name, cost, card_type, effect, targets); alt-cost mutual exclusion; aftermath cost/effect/target retrieval helpers; 12 unit tests in tests/aftermath.rs; game script pending_review |
| Disturb | 702.146 | P4 | `none` | — | — | — | — | Cast transformed from graveyard |
| Unearth | 702.84 | P3 | `validated` | `rules/abilities.rs`, `rules/turn_actions.rs`, `rules/replacement.rs`, `rules/resolution.rs`, `state/types.rs`, `state/stack.rs`, `cards/card_definition.rs` | Dregscape Zombie | `stack/087` | — | Full flow: activate from graveyard (sorcery speed), return to battlefield w/ haste, exile at end step delayed trigger, zone-change replacement (leave BF -> exile). 12 unit tests in `unearth.rs` |
| Embalm | 702.128 | P4 | `validated` | `state/types.rs:801` (KeywordAbility::Embalm disc 92), `state/hash.rs:519-520`, `cards/card_definition.rs:279-287` (AbilityDefinition::Embalm{cost}), `rules/command.rs:432-442` (Command::EmbalmCard), `rules/abilities.rs:1054-1235` (handle_embalm_card + get_embalm_cost), `rules/resolution.rs:2230-2304` (EmbalmAbility resolution: token copy w/ white + no mana cost + Zombie subtype), `rules/engine.rs:349-354`, `state/stack.rs:620-630` (StackObjectKind::EmbalmAbility disc 27), `testing/replay_harness.rs:545-550` (embalm_card action) | Sacred Cat (`defs/sacred_cat.rs`) | `stack/129` | — | CR 702.128a-b fully enforced; activated ability from graveyard (sorcery speed); exile card as cost (CR 602.2c); pay embalm mana cost; EmbalmAbility on stack; resolution creates token copy: white, no mana cost, Zombie in addition to original types, retains printed abilities; split second blocks activation; not a cast (no SpellCast event); summoning sickness on token; 12 unit tests in `tests/embalm.rs`; game script pending_review |
| Eternalize | 702.129 | P4 | `validated` | `state/types.rs:809` (KeywordAbility::Eternalize disc 93), `state/hash.rs:521-522`, `cards/card_definition.rs:288-296` (AbilityDefinition::Eternalize{cost} disc 26), `rules/command.rs:444-454` (Command::EternalizeCard), `rules/abilities.rs:1246-1435` (handle_eternalize_card + get_eternalize_cost), `rules/resolution.rs:2408-2487` (EternalizeAbility resolution: token copy w/ black + 4/4 + no mana cost + Zombie subtype), `rules/engine.rs:364-365`, `state/stack.rs:633-645` (StackObjectKind::EternalizeAbility disc 28), `testing/replay_harness.rs:556-564` (eternalize_card action) | Proven Combatant (`defs/proven_combatant.rs`) | `stack/130` | — | CR 702.129a fully enforced; activated ability from graveyard (sorcery speed); exile card as cost (CR 602.2c); pay eternalize mana cost; EternalizeAbility on stack; resolution creates token copy: black, 4/4, no mana cost, Zombie in addition to original types, retains printed abilities; split second blocks activation; not a cast (no SpellCast event); summoning sickness on token; 12 unit tests in `tests/eternalize.rs`; game script pending_review |
| Ninjutsu | 702.49 | P4 | `validated` | `state/types.rs:753` (KeywordAbility::Ninjutsu disc 87), `state/hash.rs:509-510`, `cards/card_definition.rs:271` (AbilityDefinition::Ninjutsu{cost}), `rules/command.rs:413` (Command::ActivateNinjutsu), `rules/abilities.rs:804` (handle_ninjutsu: 14-check validation), `rules/resolution.rs:2081` (resolve_ninjutsu: full ETB site + combat registration), `state/stack.rs:597` (StackObjectKind::NinjutsuAbility disc 26), `testing/replay_harness.rs:525` | Ninja of the Deep Hours (`defs/ninja_of_the_deep_hours.rs`) | `combat/125` | — | CR 702.49a-c fully enforced; activated from hand during DeclareBlockers; pay cost + return unblocked attacker to owner's hand; ninja enters tapped and attacking same target; not declared as attacker (no attack triggers); split second blocks activation; blocked attacker rejected; owner-not-controller return (multiplayer); 12 unit tests in `tests/ninjutsu.rs`; game script pending_review |
| Commander Ninjutsu | 702.49d | P4 | `validated` | `state/types.rs:761` (KeywordAbility::CommanderNinjutsu disc 88), `state/hash.rs:511-512`, `cards/card_definition.rs:278` (AbilityDefinition::CommanderNinjutsu{cost}), `rules/abilities.rs:858-888` (command zone detection + keyword check) | — (test-only card in `tests/ninjutsu.rs`) | — | Ninjutsu | CR 702.49d variant; also functions from command zone; no commander tax increment; shares ActivateNinjutsu command + NinjutsuAbility resolution with Ninjutsu; 1 dedicated unit test (test_commander_ninjutsu_from_command_zone) in `tests/ninjutsu.rs` |
| Plot | 702.170 | P4 | `validated` | `state/types.rs:529` (KeywordAbility::Plot disc 97), `state/hash.rs:529-530`, `state/game_object.rs:495-502` (is_plotted, plotted_turn), `state/stack.rs:167-172` (StackObject::was_plotted), `cards/card_definition.rs:351-358` (AbilityDefinition::Plot{cost} disc 30), `rules/plot.rs` (special action handler: priority/turn/main-phase/empty-stack checks, cost payment, exile face-up, CardPlotted event), `rules/casting.rs:81,202,296,880,948` (AltCostKind::Plot free-cast path: sorcery-speed, zero cost, mutual exclusion with 14 other alt costs, plotted_turn < current_turn enforcement), `rules/command.rs:354-360` (Command::PlotCard), `rules/engine.rs:22,314-316` (plot module + PlotCard dispatch), `rules/events.rs:778-785` (GameEvent::CardPlotted), `testing/replay_harness.rs:714-740,1364-1376` (plot_card + cast_spell_plot + find_plotted_in_exile) | Slickshot Show-Off (`defs/slickshot_show_off.rs`) | `stack/134` | — | CR 702.170a-f fully enforced; special action from hand (CR 116.2k, does NOT use the stack); pay plot cost and exile card face-up; is_plotted + plotted_turn on GameObject; cast from exile on any later turn without paying mana cost (CR 702.170d); sorcery-speed timing for both plot action and free cast; mutual exclusion with flashback/evoke/bestow/madness/miracle/escape/foretell/overload/retrace/jump-start/aftermath/dash/blitz/ninjutsu; 20 unit tests in `tests/plot.rs`; game script stack/134 (Slickshot Show-Off plotted turn 1, cast free turn 3) |
| Blitz | 702.152 | P4 | `validated` | `state/types.rs:110` (KeywordAbility::Blitz disc 96), `state/hash.rs:527-528`, `cards/card_definition.rs:342-350` (AbilityDefinition::Blitz{cost} disc 29), `rules/casting.rs:80,782-848` (blitz alt-cost validation + mutual exclusion with 12 other alt costs + get_blitz_cost), `rules/casting.rs:935-938` (blitz cost payment), `rules/casting.rs:1500-1501` (was_blitzed transfer to stack), `rules/casting.rs:1922-1933` (get_blitz_cost helper), `rules/resolution.rs:290-315` (ETB haste grant + draw-on-death trigger wiring for was_blitzed), `rules/resolution.rs:1078-1087` (BlitzSacrificeTrigger resolution: sacrifice or fizzle if gone), `rules/turn_actions.rs:255-274` (end_step_actions: BlitzSacrificeTrigger queue for blitzed permanents), `state/stack.rs:159-166` (StackObject::was_blitzed), `state/stack.rs:712-727` (StackObjectKind::BlitzSacrificeTrigger disc 32), `state/stubs.rs:70-71` (PendingTriggerKind::BlitzSacrifice), `testing/replay_harness.rs:824-837` (cast_spell_blitz action) | Riveteers Requisitioner (`defs/riveteers_requisitioner.rs`) | `stack/133` | — | CR 702.152a fully enforced; three abilities per CR text: (1) alt cost on stack, (2) haste + "When this creature dies, draw a card" while on battlefield, (3) delayed sacrifice trigger at beginning of next end step; blitz is an alternative cost (CR 118.9) -- mutual exclusion with flashback/evoke/bestow/madness/miracle/escape/foretell/overload/retrace/jump-start/aftermath/dash; commander tax applies on top of blitz cost (CR 118.9d); creature leaving battlefield before trigger resolves causes fizzle (CR 400.7); copies do not inherit was_blitzed; 9 unit tests in `tests/blitz.rs`; game script stack/133 (Riveteers Requisitioner cast with blitz, attacks, sacrificed at end step, both death triggers fire: blitz draw + WhenDies Treasure) |
| Dash | 702.109 | P4 | `validated` | `state/types.rs:819` (KeywordAbility::Dash disc 95), `state/hash.rs:525-526`, `cards/card_definition.rs:334-341` (AbilityDefinition::Dash{cost} disc 28), `rules/command.rs:200-203` (CastSpell::cast_with_dash), `rules/casting.rs:708-768` (dash alt-cost validation + mutual exclusion with 11 other alt costs), `rules/casting.rs:851-854` (dash cost lookup + payment), `rules/casting.rs:1435-1436` (was_dashed transfer to stack), `rules/casting.rs:1855-1866` (get_dash_cost helper), `rules/resolution.rs:286-290` (ETB haste grant: was_dashed transfer + Haste keyword), `rules/turn_actions.rs:270-332` (end_step_actions: DashReturnTrigger queue for was_dashed permanents), `rules/resolution.rs:1011-1017` (DashReturnTrigger resolution: return to hand or fizzle if gone), `state/game_object.rs:487-494` (GameObject::was_dashed), `state/stack.rs:152-158` (StackObject::was_dashed), `state/stack.rs:688-703` (StackObjectKind::DashReturnTrigger disc 31), `state/stubs.rs:355-361` (PendingTrigger::is_dash_return_trigger), `testing/replay_harness.rs:907-932` (cast_spell_dash action) | Zurgo Bellstriker (`defs/zurgo_bellstriker.rs`) | `stack/132` | — | CR 702.109a fully enforced; three abilities per CR text: (1) alt cost on stack, (2) delayed return trigger at next end step, (3) haste while was_dashed; dash is an alternative cost (CR 118.9) -- mutual exclusion with flashback/evoke/bestow/madness/miracle/escape/foretell/buyback/overload/retrace/jump-start/aftermath; commander tax applies on top of dash cost (CR 118.9d); creature leaving battlefield before trigger resolves causes fizzle (CR 400.7); copies do not inherit was_dashed; 7 unit tests in `tests/dash.rs`; game script stack/132 (Zurgo Bellstriker cast with dash, attacks, returns at end step) |

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
| Assist | 702.132 | P4 | `validated` | `state/types.rs:L951`, `rules/casting.rs:L72-73+L2172-2287` | Huddle Up | `stack/142` | — | KeywordAbility::Assist disc 105; assist_player + assist_amount on CastSpell; validation + payment in casting.rs; 11 unit tests in assist.rs |
| Surge | 702.117 | P4 | `validated` | `rules/casting.rs`, `rules/resolution.rs`, `state/types.rs`, `state/stack.rs` | Reckless Bushwhacker | `stack/140` | — | Alt cost (CR 118.9a mutual exclusion); `spells_cast_this_turn` precondition; 11 unit tests in surge.rs |
| Spectacle | 702.137 | P4 | `validated` | `rules/casting.rs`, `effects/mod.rs`, `rules/combat.rs`, `rules/turn_actions.rs`, `state/player.rs` | Skewer the Critics | `stack/139` | — | Alt cost (CR 118.9a mutual exclusion); `life_lost_this_turn` on PlayerState; 11 unit tests in spectacle.rs |
| Emerge | 702.119 | P4 | `validated` | `state/types.rs:113-114` (AltCostKind::Emerge), `state/types.rs:906-912` (KeywordAbility::Emerge), `state/hash.rs:537-538,3236-3237` (disc 101/33), `cards/card_definition.rs:392-399` (AbilityDefinition::Emerge{cost}), `rules/command.rs:176-185` (CastSpell::emerge_sacrifice), `rules/casting.rs:86,1105-1253,1372-1380,1953-1965,3482-3508` (emerge validation + sacrifice + cost reduction + get_emerge_cost + reduce_cost_by_mv), `rules/engine.rs:96,116` (emerge_sacrifice dispatch), `testing/replay_harness.rs:253-256,1004-1028` (cast_spell_emerge action), `testing/script_schema.rs:286-291` (emerge_sacrifice field) | Elder Deep-Fiend (`defs/elder_deep_fiend.rs`) | `stack/138` | — | CR 702.119a fully enforced; alternative cost (CR 118.9) -- pay emerge cost and sacrifice a creature; total cost reduced by sacrificed creature's MV (CR 702.119b); reduce_cost_by_mv reduces generic first; sacrifice validation: must be creature, on battlefield, controlled by caster; mutual exclusion with 14 other alt costs (flashback/evoke/bestow/madness/miracle/escape/foretell/overload/retrace/jump-start/aftermath/dash/blitz/plot/impending); sacrifice at cost-payment time (CR 601.2f-h); `cast_spell_emerge` harness action with `emerge_sacrifice` field; 10 unit tests in `tests/emerge.rs`; game script `pending_review` (Elder Deep-Fiend cast via emerge sacrificing Scroll Thief MV 3, cost {5}{U}{U} reduced to {2}{U}{U}) |
| Bargain | 702.166 | P4 | `validated` | `state/types.rs:891-903` (KeywordAbility::Bargain disc 100), `cards/card_definition.rs:950-955` (Condition::WasBargained), `rules/casting.rs:1368-1407+1750-1832` (bargain validation + sacrifice + was_bargained flag), `rules/resolution.rs:208+281` (bargain status propagation), `effects/mod.rs:70-73+2776-2777` (EffectContext::was_bargained + Condition eval), `state/stack.rs:193-201` (StackObject::was_bargained), `state/game_object.rs:518-523` (GameObject::was_bargained), `rules/command.rs:164-174` (CastSpell::bargain_sacrifice), `rules/copy.rs:211+403` (copy propagation + cascade exclusion), `testing/replay_harness.rs:249+956-958` (cast_spell_bargain action), `testing/script_schema.rs:280` | Torch the Tower | `stack/137` | — | KeywordAbility::Bargain disc 100; Condition::WasBargained disc 10; optional additional cost (CR 118.8), not alternative; CastSpell.bargain_sacrifice: Option<ObjectId>; validates keyword present + target on battlefield + controlled by caster + is artifact/enchantment/token (CR 702.166a); was_bargained propagated StackObject→GameObject→EffectContext; copies of bargained spell also bargained (CR 707.2); cascade/suspend free casts not bargained; 10 unit tests in `tests/bargain.rs`; `cast_spell_bargain` harness action; game script stack/137 (2 scenarios: unbargained 2 dmg, bargained 3 dmg via Torch the Tower) |

---

## Section 6: Spell & Ability Modifiers

Keywords that modify how spells are cast, copied, or resolved.

| Ability | CR | Priority | Status | Engine File(s) | Card Def | Script | Depends On | Notes |
|---------|----|----------|--------|----------------|----------|--------|------------|-------|
| Storm | 702.40 | P1 | `validated` | `rules/casting.rs`, `rules/copy.rs` | Grapeshot (etc.) | `stack/` scripts | — | Copy for each prior spell this turn |
| Cascade | 702.85 | P1 | `validated` | `rules/casting.rs`, `rules/copy.rs` | Bloodbraid Elf (etc.) | `stack/` scripts | — | Exile until nonland with lesser MV, cast free |
| Kicker | 702.33 | P2 | `validated` | `state/types.rs:244`, `cards/card_definition.rs:152-166`, `effects/mod.rs:60-91,1744-1745`, `state/stack.rs:58`, `state/game_object.rs:269`, `rules/command.rs:88`, `rules/casting.rs:169-205,393,549-568`, `rules/resolution.rs:149-156,187`, `testing/script_schema.rs:228-232`, `testing/replay_harness.rs:204,238` | Burst Lightning, Torch Slinger | `stack/065` | — | KeywordAbility::Kicker enum + AbilityDefinition::Kicker { cost, is_multikicker }; Condition::WasKicked; kicker_times_paid on StackObject + GameObject; kicker_times on CastSpell; get_kicker_cost + validation/payment in casting.rs; kicker propagation to EffectContext in resolution.rs; harness kicked:bool support; 10 unit tests in `tests/kicker.rs`; game script pending_review (assertions pass) |
| Overload | 702.96 | P3 | `validated` | `state/types.rs:599`, `cards/card_definition.rs:255-263,821`, `state/stack.rs:127-134`, `effects/mod.rs:66-69,2759-2760`, `rules/command.rs:157-166`, `rules/casting.rs:485-529,591-597,737-743,974-975`, `rules/resolution.rs:179-186`, `testing/replay_harness.rs:668-689` | Vandalblast | `baseline/108` | — | KeywordAbility::Overload enum + AbilityDefinition::Overload { cost }; Condition::WasOverloaded; cast_with_overload on CastSpell command; was_overloaded on StackObject + EffectContext; overload cost payment as alternative cost (CR 118.9); no-targets enforcement (CR 702.96b); alternative cost mutual exclusion (flashback/evoke/bestow/madness/miracle/escape/foretell); commander tax stacking; harness cast_spell_overload action; 11 unit tests in `tests/overload.rs`; game script baseline/108 approved |
| Replicate | 702.56 | P4 | `validated` | `rules/casting.rs`, `rules/resolution.rs`, `state/types.rs`, `state/stack.rs` | Train of Thought | `stack/143` | — | Pay replicate cost N times → N copies; 6 unit tests in `replicate.rs` |
| Splice | 702.47 | P4 | `validated` | `rules/casting.rs`, `rules/resolution.rs`, `state/types.rs`, `state/stack.rs`, `state/hash.rs`, `rules/command.rs`, `cards/card_definition.rs` | Glacial Ray, Reach Through Mists | `stack/146` | — | splice_cards on CastSpell; spliced_effects+spliced_card_ids on StackObject; 9 unit tests in `splice.rs` |
| Entwine | 702.42 | P4 | `validated` | `state/types.rs`, `cards/card_definition.rs`, `state/hash.rs`, `rules/command.rs`, `state/stack.rs`, `rules/casting.rs`, `rules/resolution.rs`, `testing/replay_harness.rs` | Promise of Power | `stack/147` | — | KW disc 110, AbilDef disc 39; entwine_paid on CastSpell, was_entwined on StackObject; ModeSelection added to helpers.rs exports; 6 unit tests in `entwine.rs` |
| Fuse | 702.102 | P4 | `none` | — | — | — | — | Cast both halves of a split card |
| Buyback | 702.27 | P3 | `validated` | `state/types.rs:497-503`, `state/stack.rs:118`, `rules/casting.rs:67,597-625,894,1113-1129`, `rules/resolution.rs:489-490` | Searing Touch | `stack/094` | — | KeywordAbility::Buyback enum + AbilityDefinition::Buyback { cost }; cast_with_buyback param on CastSpell command; get_buyback_cost lookup in casting.rs; was_buyback_paid on StackObject; resolution destination check in resolution.rs (CR 702.27a); flashback exile overrides buyback (CR 702.34a); 9 unit tests in `tests/buyback.rs` covering basic payment, cost aggregation, counter interaction, fizzle behavior; game script stack/094 pending_review (assertions pass) |
| Spree | 702.172 | P4 | `none` | — | — | — | — | Choose modes, pay cost for each |
| Cleave | 702.148 | P4 | `validated` | `state/types.rs`, `rules/casting.rs`, `rules/resolution.rs`, `effects/mod.rs` | Path of Peril | `stack/145` | — | AltCostKind::Cleave + Condition::WasCleaved; 8 unit tests |
| Escalate | 702.120 | P4 | `validated` | `state/types.rs`, `cards/card_definition.rs`, `state/hash.rs`, `rules/command.rs`, `state/stack.rs`, `rules/casting.rs`, `rules/resolution.rs`, `testing/replay_harness.rs` | Blessed Alliance | `stack/148` | — | KW disc 111, AbilDef disc 40 (Escalate { cost }); escalate_modes on CastSpell, escalate_modes_paid on StackObject; CR 702.120a additional cost validation; 9 unit tests in `tests/escalate.rs` |
| Split Second | 702.61 | P2 | `validated` | `state/types.rs:250-255`, `state/hash.rs:347-348`, `rules/casting.rs:66-72,1060-1074`, `rules/abilities.rs:63-70,388-395` | Krosan Grip | `stack/066` | — | KeywordAbility::SplitSecond enum + has_split_second_on_stack helper; CastSpell gate (CR 702.61a), ActivateAbility gate (CR 702.61a), CycleCard gate (CR 702.61a); mana abilities exempt (CR 702.61b); triggered abilities still fire (CR 702.61b); uses calculate_characteristics for layer-aware keyword check; 8 unit tests in `tests/split_second.rs`; game script pending_review |
| Gravestorm | 702.69 | P4 | `validated` | types.rs, casting.rs, resolution.rs, stack.rs, hash.rs | Follow the Bodies | stack/144 | Storm variant; permanents_put_into_graveyard_this_turn counter; 9 unit tests |

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
| Enlist | 702.154 | P4 | `validated` | `state/types.rs:734-745`, `state/stack.rs:577-582`, `state/stubs.rs:326-339`, `state/combat.rs:65-70`, `state/builder.rs:532-552`, `state/hash.rs:507-508+1501-1502`, `rules/combat.rs:263-417`, `rules/abilities.rs:1524-1572+2983-2989`, `rules/resolution.rs:2006-2060` | Coalition Skyknight (`defs/coalition_skyknight.rs`) | `combat/124` | — | KeywordAbility::Enlist enum (discriminant 86); StackObjectKind::EnlistTrigger (discriminant 25) with source_object+enlisted_creature; enlist_choices on DeclareAttackers command; 10-check validation in combat.rs (self-enlist CR 702.154c, summoning sickness, haste override, tapped check, creature type check, attacker exclusion, controller check, instance count CR 702.154d, already-enlisted uniqueness, battlefield check); CombatState.enlist_pairings; linked trigger post-processing in abilities.rs (matches placeholders to pairings, removes unused Enlist placeholders); resolution.rs reads enlisted creature power via calculate_characteristics and applies +X/+0 UntilEndOfTurn; CR 702.154b linked ability; 8 unit tests in `tests/enlist.rs` (basic power addition, no-choice-no-trigger, enlisted-must-not-be-attacking, summoning-sickness-rejected, haste-allowed, cannot-enlist-self, creature-used-once-only, multiplayer-4-player); card def coalition_skyknight.rs (3W 2/2 Human Knight, Flying+Enlist); game script combat/124 validated |
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
| Encore | 702.141 | P4 | `validated` | `state/types.rs:810-818` (KeywordAbility::Encore, discriminant 94), `state/game_object.rs:451-486` (encore_sacrifice_at_end_step, encore_must_attack, encore_activated_by), `state/hash.rs:523-524+692-697+1131-1134+1563-1573+3160-3161`, `state/stack.rs:649-677` (StackObjectKind::EncoreAbility disc 29, EncoreSacrificeTrigger disc 30), `state/stubs.rs:340-354` (is_encore_sacrifice_trigger, encore_activator on PendingTrigger), `state/builder.rs:920-922`, `state/mod.rs:300-305+397-402` (CR 400.7 zone-change reset), `cards/card_definition.rs:297-306` (AbilityDefinition::Encore { cost } disc 27), `rules/command.rs:456-469` (Command::EncoreCard), `rules/engine.rs:379-384` (dispatch to handle_encore_card), `rules/abilities.rs:1448-1628` (handle_encore_card 14-check validation + get_encore_cost; sorcery-speed, graveyard zone, keyword check, split second block, mana cost payment, exile as cost, StackObjectKind::EncoreAbility push), `rules/abilities.rs:3871-3876` (flush: EncoreSacrificeTrigger kind), `rules/resolution.rs:2598-2733` (EncoreAbility resolution: for-each-opponent token creation, haste, encore_sacrifice_at_end_step/encore_must_attack/encore_activated_by tags), `rules/resolution.rs:2799-2808` (EncoreSacrificeTrigger resolution: controller check + sacrifice), `rules/turn_actions.rs:192-259` (end_step_actions: queue EncoreSacrificeTrigger for encore tokens), `testing/replay_harness.rs:588-593` (encore_card action type), `tools/replay-viewer/src/view_model.rs:508-512+745`, `tools/tui/src/play/panels/stack_view.rs:115-120` | Briarblade Adept (`briarblade_adept.rs`) | `tests/encore.rs` (10 tests) | `stack/131` | CR 702.141a fully enforced. Activated ability from graveyard (sorcery speed); exile card as cost (CR 602.2c); EncoreAbility on stack; resolution creates one token copy per living opponent: tokens gain Haste, tagged encore_sacrifice_at_end_step=true + encore_must_attack=Some(opponent_id) + encore_activated_by=Some(activator); end_step_actions queues EncoreSacrificeTrigger (sacrifice only if controller==activator); CR 400.7 zone-change resets; split second blocks; not a cast; eliminated opponents skipped; 10 unit tests cover 4p basic, 2p, haste, exile-as-cost, sacrifice-at-end-step, sorcery-speed (opponent turn, non-empty stack), not-in-graveyard, no-keyword, eliminated-opponent; game script pending_review. Attack trigger on Briarblade Adept omitted (targeted_trigger DSL gap). |
| Champion | 702.72 | P4 | `validated` | `state/types.rs` (KW 126, ChampionFilter), `cards/card_definition.rs` (AbilityDefinition::Champion disc 49), `state/game_object.rs` (champion_exiled_card), `state/stubs.rs` (PendingTriggerKind::ChampionETB/LTB), `state/stack.rs` (StackObjectKind::ChampionETBTrigger disc 47, ChampionLTBTrigger disc 48), `rules/resolution.rs` (ETB exile-or-sacrifice + LTB return) | Changeling Hero | `stack/164` | — | CR 702.72a ETB exile qualifying creature or sacrifice self; CR 702.72b LTB return exiled card to battlefield under owner's control; CR 607.2 linked abilities; champion_exiled_card on GameObject; LTB wired to CreatureDied/PermanentDestroyed/ObjectExiled/ObjectReturnedToHand events; 9 unit tests in `tests/champion.rs` |
| Devour | 702.82 | P4 | `validated` | `rules/resolution.rs`, `state/types.rs` (KW 124), `testing/replay_harness.rs` | Predator Dragon | `stack/162` | — | ETB replacement: optional sacrifice → +1/+1 counters (702.82a-c); devour_sacrifices on CastSpell/StackObject; creatures_devoured tracking; 10 unit tests |
| Tribute | 702.104 | P4 | `none` | — | — | — | — | Opponent chooses: +1/+1 counters or ability triggers |
| Fabricate | 702.123 | P4 | `none` | — | — | — | — | ETB: choose +1/+1 counters or create Servo tokens |
| Decayed | 702.147 | P4 | `validated` | state/types.rs:616-628, state/game_object.rs:399-408, rules/combat.rs:285-300+396-401, rules/turn_actions.rs:606-632 | Shambling Ghast | decayed.rs (8 tests) | baseline/112 | Can't block; sacrifice at end of combat when it attacks. Flag-on-object pattern (like Myriad); persists even if keyword removed (ruling 2021-09-24). |
| Training | 702.149 | P4 | `validated` | state/types.rs:693-700 (KeywordAbility::Training, discriminant 82), state/game_object.rs:206-211 (TriggerEvent::SelfAttacksWithGreaterPowerAlly, discriminant 19), state/builder.rs:489-511 (TriggeredAbilityDef auto-generation), state/hash.rs:493-494+1189-1190, rules/abilities.rs:1508-1550 (AttackersDeclared dispatch: layer-aware power comparison, SelfAttacksWithGreaterPowerAlly event) | Gryff Rider | training.rs (7 tests) | combat/120 | CR 702.149a+b fully enforced; triggered ability auto-generated from keyword in builder.rs; AttackersDeclared handler checks strictly-greater power among co-attackers (layer-aware); multiple instances trigger separately (CR 702.149b); 7 unit tests cover basic trigger, attacking alone (negative), equal power (negative), lower power (negative), multiple instances, two training creatures, 4-player multiplayer; game script approved |
| Backup | 702.165 | P4 | `validated` | state/types.rs:1144-1152 (KeywordAbility::Backup(u32), discriminant 125), state/stubs.rs:93-96+307-314 (PendingTriggerKind::Backup, backup_abilities, backup_n), state/stack.rs:990-1001 (StackObjectKind::BackupTrigger, discriminant 46), rules/abilities.rs:2276-2320 (ETB trigger detection, ability snapshot CR 702.165d), rules/abilities.rs:4499-4506 (flush arm), rules/resolution.rs:2741-2745 (+counters + Layer 6 ability grant until EOT) | Backup Agent | backup.rs (11 tests) | stack/163 | CR 702.165a ETB trigger, counters + ability grant; 702.165d abilities snapshotted at trigger time; multiple instances work separately |

---

## Section 9: Counters & Growth

Keywords involving counter manipulation and creature growth.

| Ability | CR | Priority | Status | Engine File(s) | Card Def | Script | Depends On | Notes |
|---------|----|----------|--------|----------------|----------|--------|------------|-------|
| Modular | 702.43 | P3 | `validated` | `state/types.rs`, `state/hash.rs`, `state/stack.rs:269`, `state/stubs.rs`, `state/builder.rs:641`, `rules/resolution.rs:309+ModularTrigger`, `rules/abilities.rs` | Arcbound Worker | `stack/092` | — | CR 702.43a fully enforced; ETB counter placement (static) + dies trigger (targeted, dynamic counter count from pre_death_counters); StackObjectKind::ModularTrigger; auto-targets first artifact creature; 9 unit tests in modular.rs; script pending_review |
| Graft | 702.58 | P4 | `validated` | `state/types.rs:1096`, `rules/abilities.rs:2415`, `rules/resolution.rs:640+2234`, `state/stack.rs`, `state/stubs.rs`, `state/hash.rs` | Simic Initiate | `stack/156` | — | CR 702.58a fully enforced; static ETB with N +1/+1 counters + triggered ability (intervening-if: has +1/+1 counter, may move one to entering creature); StackObjectKind::GraftTrigger; PendingTriggerKind::Graft; KeywordAbility::Graft(u32) disc 119; 9 unit tests in graft.rs; script validated |
| Evolve | 702.100 | P3 | `validated` | `state/types.rs:496`, `state/hash.rs:429`, `state/hash.rs:1244`, `state/stubs.rs:157`, `rules/abilities.rs:900`, `rules/resolution.rs:1015` | Cloudfin Raptor | `stack/093` | — | CR 702.100a fully enforced; intervening-if re-checked at resolution (CR 603.4); ETB creature comparison (P > P and/or T > T); StackObjectKind::EvolveTrigger; uses last-known P/T if entering creature leaves battlefield (ruling 2013-04-15); 10 unit tests in evolve.rs; script pending_review |
| Scavenge | 702.97 | P4 | `validated` | `state/types.rs:1112`, `cards/card_definition.rs:525`, `rules/abilities.rs:4847`, `rules/engine.rs:432`, `rules/resolution.rs`, `rules/command.rs:552`, `state/stack.rs:978`, `state/hash.rs` | Deadbridge Goliath | `stack/157` | — | CR 702.97a fully enforced; activated ability from graveyard, sorcery-speed, pay cost + exile card, snapshot power, put +1/+1 counters on target creature; KeywordAbility::Scavenge disc 120; AbilityDefinition::Scavenge disc 47; StackObjectKind::ScavengeAbility disc 45; 10 unit tests in scavenge.rs; script validated |
| Outlast | 702.107 | P4 | `validated` | `state/types.rs:1117`, `cards/card_definition.rs:526`, `testing/replay_harness.rs:1711`, `state/hash.rs:612,3628` | Ainok Bond-Kin | `stack/158` | — | CR 702.107a fully enforced; AbilityDefinition::Outlast{cost} expands into ActivatedAbility (tap + mana cost, sorcery speed, +1/+1 counter on self) via enrich_spec_from_def; KeywordAbility::Outlast disc 121; AbilityDefinition::Outlast disc 48; no custom StackObjectKind (uses generic ActivatedAbility path); 7 unit tests in outlast.rs; script validated |
| Amplify | 702.38 | P4 | `validated` | `state/types.rs:1126`, `rules/resolution.rs:679`, `state/hash.rs:614` | Canopy Crawler | `stack/159` | — | CR 702.38a fully enforced; as-enters replacement effect, reveal creature cards sharing a creature type from hand, N +1/+1 counters per card revealed; KeywordAbility::Amplify(u32) disc 122; implemented in resolution.rs ETB path; 8 unit tests in `amplify.rs`; script validated |
| Adapt | 701.46 | P3 | `validated` | `state/types.rs:560`, `cards/card_definition.rs:793`, `effects/mod.rs:2731`, `state/hash.rs:445,2431` | Sharktocrab | `baseline/105` | — | CR 701.46a fully enforced; keyword action (not keyword ability); KeywordAbility::Adapt(u32) enum + Condition::SourceHasNoCountersOfType + Conditional effect; resolution-time check per ruling 2019-01-25 (activation always legal); re-adapt after losing counters verified; 6 unit tests in `adapt.rs`; game script approved |
| Bolster | 701.39 | P3 | `validated` | `cards/card_definition.rs:388`, `effects/mod.rs:971` | Cached Defenses | `baseline/104` | — | CR 701.39a fully enforced; keyword action (not keyword ability); chooses creature with least layer-aware toughness (ruling 2014-11-24); does NOT target (protection irrelevant); deterministic tie-break by smallest ObjectId; no-op if controller has no creatures; 8 unit tests in `bolster.rs`; game script approved |

---

## Section 10: Upkeep, Time & Phasing

Keywords involving time-based effects, phasing, and recurring costs.

| Ability | CR | Priority | Status | Engine File(s) | Card Def | Script | Depends On | Notes |
|---------|----|----------|--------|----------------|----------|--------|------------|-------|
| Cycling | 702.29 | P2 | `validated` | `state/types.rs:195`, `cards/card_definition.rs:145`, `state/hash.rs:316+1630+2220`, `rules/command.rs:182`, `rules/engine.rs:185`, `rules/abilities.rs:365`, `rules/events.rs:386` | Lonely Sandbar | `stack/061` | — | KeywordAbility::Cycling enum + AbilityDefinition::Cycling { cost }; Command::CycleCard dispatch; handle_cycle_card validates zone/keyword/mana, discards as cost, pushes draw onto stack; GameEvent::CardCycled emitted; 12 unit tests in `tests/cycling.rs`; game script approved |
| Suspend | 702.62 | P3 | `validated` | `state/types.rs:543`, `cards/card_definition.rs:252`, `rules/suspend.rs` (197 lines), `rules/command.rs:356`, `rules/events.rs:784`, `rules/engine.rs:304`, `rules/turn_actions.rs:35-91`, `rules/abilities.rs`, `rules/resolution.rs:1278-1449`, `state/hash.rs:439+2086` | Rift Bolt (#111) | `stack/102` | — | KeywordAbility::Suspend enum + AbilityDefinition::Suspend { cost, time_counters }; Command::SuspendCard special action (CR 116.2f); handle_suspend_card validates zone/keyword/timing/mana, exiles face-up with N time counters, is_suspended=true; GameEvent::CardSuspended (hash 86); upkeep trigger dispatch in turn_actions.rs queues SuspendCounterTrigger; resolution.rs removes counter, queues SuspendCastTrigger when last removed; cast trigger casts without paying mana cost (CR 702.62d), creatures gain haste (CR 702.62a); multiplayer: only owner's upkeep ticks; 9 unit tests in `tests/suspend.rs`; game script approved |
| Phasing | 702.26 | P4 | `validated` | `rules/turn_actions.rs:L672-L815`, `rules/sba.rs:L643,L758`, `rules/layers.rs:L216-L220`, `rules/combat.rs:L78,L519`, `effects/mod.rs:L2522`, `rules/events.rs:L961-L978`, `state/types.rs:L1082-L1095` | Teferi's Isle | `stack/155` | — | CR 702.26a-h: phase-in/phase-out simultaneously before untap (CR 502.1); phased-out permanents treated as nonexistent (filters in sba.rs, layers.rs, combat.rs); no zone change (CR 702.26d — no ETB/LTB triggers); indirect phasing for attachments (CR 702.26h); GameEvent::PermanentsPhasedOut/PermanentsPhasedIn; 16 unit tests in `tests/phasing.rs`; game script approved |
| Cumulative Upkeep | 702.24 | P4 | `validated` | `rules/turn_actions.rs:L319`, `rules/resolution.rs:L1411,L4022`, `rules/engine.rs:L453`, `rules/abilities.rs:L3851` | Mystic Remora | `stack/152` | — | CR 702.24a-b: upkeep trigger adds age counter then pay cost*age or sacrifice; CumulativeUpkeepCost enum (Mana/Life); CumulativeUpkeepTrigger StackObjectKind; Command::PayCumulativeUpkeep; 8 unit tests in `tests/cumulative_upkeep.rs`; game script approved |
| Echo | 702.30 | P4 | `validated` | `rules/turn_actions.rs:L239`, `rules/resolution.rs:L502,L1357`, `rules/lands.rs:L179`, `rules/engine.rs:L429`, `rules/abilities.rs:L3826` | Avalanche Riders | `stack/151` | — | CR number corrected 702.31->702.30; KeywordAbility::Echo(ManaCost) enum; ETB sets echo_pending (resolution.rs + lands.rs); upkeep trigger queues EchoUpkeep (turn_actions.rs); resolution emits EchoPaymentRequired; Command::PayEcho handles pay/sacrifice (engine.rs); intervening-if checks layer-resolved characteristics; 9 unit tests in `tests/echo.rs`; game script approved |
| Fading | 702.32 | P4 | `validated` | `rules/turn_actions.rs:L168`, `rules/resolution.rs:L477,L1206`, `rules/lands.rs:L148`, `rules/abilities.rs:L3803` | Blastoderm | `stack/150` | — | ETB with fade counters, upkeep removal, sacrifice at 0; 8 unit tests in `tests/fading.rs`; CR 702.32a covered; uses fade counters (not time counters like Vanishing) |
| Vanishing | 702.63 | P4 | `validated` | `rules/turn_actions.rs:L100`, `rules/resolution.rs:L449,L983,L1072`, `rules/lands.rs:L115`, `rules/abilities.rs:L3785` | Aven Riftwatcher | `stack/149` | — | ETB counters, upkeep removal, sacrifice at 0; 8 unit tests in `tests/vanishing.rs`; CR 702.63a-c covered |
| Forecast | 702.57 | P4 | `validated` | `rules/abilities.rs:L679`, `rules/engine.rs:L242`, `rules/resolution.rs:L827`, `state/mod.rs:L164` | Sky Hussar | `stack/154` | — | Activated from hand during owner's upkeep; once per turn (CR 702.57b); 9 unit tests in `tests/forecast.rs` |
| Recover | 702.59 | P4 | `validated` | `rules/abilities.rs:L554,L648`, `rules/resolution.rs:L1468`, `rules/engine.rs:L453,L882`, `testing/replay_harness.rs:L665` | Grim Harvest | `stack/153` | — | CreatureDied trigger wiring in abilities.rs; RecoverTrigger resolution + pay/exile in resolution.rs; PayRecover command in engine.rs; pay_recover harness action; 8 unit tests in `tests/recover.rs`; CR 702.59a + CR 400.7 covered |
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
| Manifest | 701.40 | P4 | `none` | — | — | — | Morph | Put top card face-down as 2/2; turn up if creature |
| Cloak | 701.58 | P4 | `none` | — | — | — | Manifest | Manifest variant with ward {2} |
| Mutate | 702.140 | P3 | `none` | — | — | — | — | Merge with creature; deferred (corner case audit) |
| Changeling | 702.73 | P2 | `validated` | `state/types.rs:286-293` (KeywordAbility::Changeling + ALL_CREATURE_TYPES:296-376), `state/hash.rs:360-361`, `state/continuous_effect.rs:139-145` (AddAllCreatureTypes), `rules/layers.rs:61-76` (inline CDA check + apply arm:326-334), `tools/replay-viewer/src/view_model.rs:602` | Universal Automaton | `layers/072` | — | CR 702.73a CDA: "This object is every creature type." Applied in Layer 4 before non-CDA effects (CR 613.3); functions in all zones (CR 604.3); ALL_CREATURE_TYPES lazy static (~290+ subtypes from CR 205.3m); LayerModification::AddAllCreatureTypes for Maskwood Nexus-style effects; 7 unit tests in `tests/changeling.rs`; game script pending_review |
| Crew | 702.122 | P2 | `validated` | `state/types.rs:302`, `rules/command.rs:245`, `rules/engine.rs:234`, `rules/abilities.rs:1246`, `testing/replay_harness.rs:408` | Smuggler's Copter | `combat/075` | 15 tests in `crew.rs`; script `pending_review` (multi-turn attack gap, same as 069/070) |
| Saddle | 702.163 | P4 | `none` | — | — | — | Crew | Crew variant for Mounts |
| Prototype | 702.160 | P4 | `validated` | `state/types.rs:882` (KeywordAbility::Prototype disc 98), `cards/card_definition.rs:373` (AbilityDefinition::Prototype{cost,power,toughness} disc 31), `rules/command.rs:163` (CastSpell.prototype: bool), `state/game_object.rs:517` (is_prototyped), `state/stack.rs:184` (was_prototyped), `rules/casting.rs:68,972-993` (prototype cost selection + stack char overwrite), `rules/resolution.rs:335-346` (ETB prototype char application), `state/mod.rs:310-316,433-438` (CR 718.4 zone-change revert), `rules/copy.rs:204-205` (CR 718.3c was_prototyped propagation), `rules/commander.rs:210-216` (prototype cost in color identity), `testing/replay_harness.rs:853` (cast_spell_prototype action) | Blitz Automaton | `stack/135` | 10 tests in `tests/prototype.rs` | CR 702.160 + CR 718; NOT an alternative cost (CR 118.9); prototype changes P/T, mana cost, color on stack+battlefield only; reverts to printed chars on zone change (CR 718.4); copies inherit was_prototyped (CR 718.3c); abilities/name/types unchanged (CR 718.5) |
| Living Metal | 702.161 | P4 | `validated` | `state/types.rs`, `rules/layers.rs` | Steel Guardian | `stack/166` | — | KW 128; Layer 4 type-change during controller's turn (CR 702.161a); 7 tests in `living_metal.rs`; all real cards are DFC Transform (blocked subsystem), uses synthetic card |
| Umbra Armor | 702.89 | P4 | `validated` | `state/types.rs`, `rules/replacement.rs`, `rules/sba.rs`, `effects/mod.rs` | Hyena Umbra | `stack/165` | Enchant | Aura destroyed instead of enchanted permanent; KW 127, disc 99; 11 tests in `umbra_armor.rs` |
| Soulbond | 702.95 | P4 | `validated` | `state/types.rs`, `state/game_object.rs`, `rules/resolution.rs`, `rules/sba.rs`, `rules/layers.rs`, `rules/abilities.rs`, `cards/card_definition.rs` | Silverblade Paladin | `stack/167` | — | KW Soulbond; `paired_with` on GameObject; SoulbondSelfETB+SoulbondOtherETB triggers; SoulbondTrigger SOK 49; SoulbondGrant+EffectDuration::WhilePaired in layers; fizzle check via calculate_characteristics; 10 tests in `soulbond.rs` |
| Haunt | 702.55 | P4 | `none` | — | — | — | — | When this dies, exile haunting a creature |
| Extort | 702.101 | P3 | `validated` | `state/types.rs:324`, `state/game_object.rs:174`, `cards/card_definition.rs:224`, `effects/mod.rs:261`, `rules/abilities.rs:640`, `state/builder.rs:561`, `state/hash.rs:376`, `tools/replay-viewer/src/view_model.rs:610` | Syndic of Tithes | `stack/078` | — | CR 702.101a+b: triggered ability on spell cast, may pay {W/B}, drain 1 from each opponent; multiple instances trigger separately; 7 unit tests in `tests/extort.rs` |
| Cipher | 702.99 | P4 | `none` | — | — | — | — | Encode spell on creature; cast copy on combat damage |
| Bloodthirst | 702.54 | P4 | `validated` | `state/types.rs:1127`, `rules/resolution.rs:779`, `state/player.rs:137`, `effects/mod.rs`, `state/hash.rs` | Stormblood Berserker | `stack/160` | — | CR 702.54a+c: ETB replacement places N +1/+1 counters if opponent was dealt damage this turn; multiple instances add independently; KeywordAbility::Bloodthirst(u32) disc 123; `damage_dealt_this_turn` on PlayerState; 8 unit tests in `tests/bloodthirst.rs` |
| Bloodrush | — | P4 | `none` | — | — | — | — | Ability word; discard to pump attacking creature |
| Devoid | 702.114 | P4 | `validated` | state/types.rs:609-615, state/hash.rs:463-464, rules/layers.rs:74-83 | Forerunner of Slaughter | baseline/111 | CR 702.114a fully enforced; CDA in Layer 5 (ColorChange) clears colors; functions in all zones (CR 604.3); 8 unit tests in devoid.rs; script approved | Colorless regardless of mana cost |
| Ingest | 702.115 | P4 | `validated` | state/types.rs:629-638, state/stubs.rs:230-242, state/stack.rs:407-427, state/hash.rs:467-468+1090-1092+1373-1374, rules/abilities.rs:1757-1840+2218-2226, rules/resolution.rs:1634-1671 | Mist Intruder | baseline/113 | CR 702.115a+b fully enforced; triggered ability on combat damage to player; multiple instances trigger separately (702.115b); empty library is safe no-op; face-up exile (default); 6 unit tests in ingest.rs; TUI stack_view.rs:80-81 + view_model.rs:473-474,687 | Combat damage to player → exile top card of library |
| Wither | 702.80 | P3 | `validated` | state/types.rs:481, state/hash.rs:422, rules/combat.rs:863-1006, effects/mod.rs:206-241 | Boggart Ram-Gang | combat/091 | CR 702.80a fully enforced; combat + non-combat damage to creatures places -1/-1 counters instead of marking damage; 6 unit tests in keywords.rs; script pending_review | Damage dealt as -1/-1 counters |
| Infect | 702.90 | P3 | `validated` | state/types.rs:511-520, state/hash.rs:433-444, rules/events.rs:357-374, rules/combat.rs:863-1073, effects/mod.rs:143-275 | Glistener Elf | combat/092 | CR 702.90 fully enforced; creature damage as -1/-1 counters (reusing Wither path); player damage as poison counters; 9 unit tests in tests/keywords.rs; poison SBA (704.5c) was pre-existing |
| Poisonous | 702.70 | P4 | `validated` | state/types.rs:722, state/hash.rs:497-500, state/stack.rs:534-554, state/stubs.rs:307-318, state/builder.rs:532-536, rules/abilities.rs:2336-2427+2890-2898, rules/resolution.rs:1967-1995 | Poisonous Viper (test card) | combat/122 | Infect poison infra | CR 702.70a+b fully enforced; triggered ability (not replacement); N is fixed, independent of damage dealt; multiple instances trigger separately (702.70b); 6 unit tests in tests/poisonous.rs; reuses PoisonCountersGiven event + 704.5c SBA from Infect infra |
| Toxic | 702.164 | P4 | `validated` | state/types.rs:723-733 (KeywordAbility::Toxic(u32), discriminant 85), state/hash.rs:502-504, rules/combat.rs:1153-1318 (inline in apply_combat_damage_assignments: total toxic value summed from all Toxic N instances, poison counters given as additional result of combat damage to a player), tools/replay-viewer/src/view_model.rs:715 | Pestilent Syphoner (`defs/pestilent_syphoner.rs`) | `combat/123` | Infect poison infra | CR 702.164a+b+c fully enforced; static ability (NOT triggered -- no StackObjectKind variant); total toxic value = sum of all Toxic N values (CR 702.164b, cumulative); combat damage to player gives poison counters equal to total toxic value as additional result (CR 702.164c); does NOT replace damage (unlike Infect); does NOT apply to creature damage; zero-power creature deals 0 damage so Toxic does not apply (CR 120.3g); Toxic + Infect coexist independently; Toxic + Lifelink coexist independently; 10-poison SBA (704.5c) tested; multiplayer-correct (only attacked player gets poison); OrdSet dedup noted as LOW limitation; 8 unit tests in tests/toxic.rs; game script combat/123 pending_review |
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
| Craft | 702.167 | P4 | `none` | — | — | — | — | Exile from battlefield + exile materials → transform |
| Connive | 701.50 | P3 | `validated` | effects/mod.rs:L1492; events.rs:L760; game_object.rs:L190; card_definition.rs:L699 | Raffine's Informant | stack/095 | 7 tests in connive.rs | CR 701.50a/e: Draw N, discard N, +1/+1 counter for each nonland discarded |
| Casualty | 702.153 | P4 | `validated` | `rules/casting.rs`, `rules/resolution.rs`, `rules/command.rs`, `state/types.rs`, `state/stack.rs`, `state/hash.rs` | Make Disappear | `stack/141` | — | CR 702.153: Optional additional cost (CR 118.8); sacrifice creature with power >= N; CasualtyTrigger copies spell; KeywordAbility::Casualty(u32) disc 104 + StackObjectKind::CasualtyTrigger disc 34; was_casualty_paid + casualty_sacrifice on CastSpell; power-threshold validation; 9 unit tests in casualty.rs |
| Alliance | — | P4 | `none` | — | — | — | — | Ability word; trigger when creature ETBs under your control |
| Ravenous | — | P4 | `none` | — | — | — | — | ETB with X +1/+1 counters; draw if X >= 5 |
| Squad | 702.157 | P4 | `none` | — | — | — | — | Pay squad cost N times → N token copies on ETB |
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
| Amass | 701.47 | P4 | `validated` | `cards/card_definition.rs:673` (Effect::Amass), `effects/mod.rs:1073-1185` (execution), `rules/events.rs:713` (GameEvent::Amassed), `state/hash.rs:2753,3347` (hash arms disc 98/41), `rules/abilities.rs:3411` (trigger dispatch) | Dreadhorde Invasion | `stack/161` | 7 tests in `amass.rs` | CR 701.47a: keyword action (no KeywordAbility variant); Effect::Amass { subtype, count }; creates 0/0 black Army token if none controlled, else adds +1/+1 counters to existing Army; adds subtype (e.g., Zombie); GameEvent::Amassed always emitted per CR 701.47b |
| Discover | 701.57 | P4 | `none` | — | — | — | Cascade | Keyword action variant of Cascade without the free cast restriction |
| Forage | 701.61 | P4 | `none` | — | — | — | — | Sacrifice a Food or exile 3 cards from graveyard |
| Offspring | 702.175 | P4 | `none` | — | — | — | — | Pay offspring cost → create 1/1 token copy on ETB |
| Impending | 702.176 | P4 | `validated` | `cards/card_definition.rs:383` (AbilityDefinition::Impending), `state/types.rs:112,890` (KeywordAbility::Impending, AltCostKind::Impending), `rules/casting.rs:83,876-1073,1190-1193,1767-1768,3171-3203` (alt cost validation, mutual exclusion, cost payment, get_impending_cost/count), `rules/resolution.rs:293,358-373,1226-1272` (ETB time counters, counter-removal trigger resolution), `rules/layers.rs:85-95` (Layer 4 type removal while impending+counters), `rules/turn_actions.rs:302-328` (end-step trigger queuing), `rules/abilities.rs:3679-3686` (trigger dispatch), `state/stack.rs:185,754-771` (was_impended, ImpendingCounterTrigger), `state/hash.rs` (hash arms disc 99/33/32), `rules/copy.rs:208,398` (cascade/copy interaction), `testing/replay_harness.rs:914-927` (cast_spell_impending action) | Overlord of the Hauntwoods | `stack/136` | — | CR 702.176a: 4 sub-abilities (alt cost, ETB time counters, Layer 4 type removal, end-step counter removal); 11 unit tests in `tests/impending.rs`; alt-cost mutual exclusion with all 14 other alt-cost keywords; commander tax interaction tested |
| Gift | 702.174 | P4 | `none` | — | — | — | — | Choose an opponent to receive a gift |
| Collect evidence | 701.59 | P4 | `none` | — | — | — | — | Exile cards from graveyard with total MV >= N |
| Suspect | 701.60 | P4 | `none` | — | — | — | — | Menace + can't block |
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

3 remaining: **Morph** (702.37, deferred), **Mutate** (702.140, deferred), **Transform** (701.28). 37/40 validated.

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

34 remaining (none), 12 n/a (digital-only + Banding + niche). 59/105 validated. Top unresolved by likely demand: Fuse, Fabricate; Morph-blocked: Disguise, Megamorph, Manifest, Cloak; Transform-blocked: Daybound/Nightbound, Disturb, Craft.

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

**Resolved**: Poisonous (CR 702.70) — validated 2026-03-01 (script combat/122, Poisonous Viper, 6 unit tests in poisonous.rs).

**Resolved**: Toxic (CR 702.164) — validated 2026-03-01 (script combat/123, Pestilent Syphoner, 8 unit tests in toxic.rs). Static ability enforced inline in combat.rs (no StackObjectKind variant). CR number corrected from 702.156 to 702.164.

**Resolved**: Enlist (CR 702.154) — validated 2026-03-01 (script combat/124, Coalition Skyknight, 8 unit tests in enlist.rs). CR number corrected from 702.155 to 702.154. Static ability (optional attack cost) + linked triggered ability; 10-check validation in combat.rs; EnlistTrigger StackObjectKind with resolution reading enlisted creature's power for +X/+0 UntilEndOfTurn.

**Resolved**: Ninjutsu (CR 702.49) + Commander Ninjutsu (CR 702.49d) — validated 2026-03-01 (script combat/125, Ninja of the Deep Hours, 12 unit tests in ninjutsu.rs). Activated ability from hand (or command zone for Commander Ninjutsu); Command::ActivateNinjutsu + NinjutsuAbility StackObjectKind; handle_ninjutsu() 14-check validation + resolve_ninjutsu() full ETB site pattern with combat registration; no commander tax for Commander Ninjutsu.

**Resolved**: Retrace (CR 702.81) — validated 2026-03-01 (script combat/126, Flame Jab, 11 unit tests in retrace.rs). Static ability on instants/sorceries; KeywordAbility::Retrace disc 89; `retrace_discard_land` on CastSpell command; additional cost (CR 118.8), not alternative; card returns to graveyard on resolution (not exile); sorcery-speed timing preserved from graveyard; `cast_spell_retrace` harness action.

**Resolved**: Jump-Start (CR 702.133) — validated 2026-03-01 (script combat/127, Radical Idea, 12 unit tests in jump_start.rs). Two static abilities: (1) cast from graveyard paying normal mana cost + discard any card as additional cost (CR 601.2b,f-h), (2) exile on any stack departure (resolve/counter/fizzle); KeywordAbility::JumpStart disc 90; `cast_with_jump_start` + `jump_start_discard` on CastSpell command; instant/sorcery type check; mutual exclusion with flashback; `cast_spell_jump_start` harness action.

**Resolved**: Aftermath (CR 702.127) — validated 2026-03-01 (script combat/128, Cut // Ribbons, 12 unit tests in aftermath.rs). Three static abilities per CR 702.127a: (1) cast aftermath half from graveyard, (2) aftermath half can only be cast from graveyard, (3) exile on stack departure; KeywordAbility::Aftermath disc 91 + AbilityDefinition::Aftermath disc 24; `cast_with_aftermath` on CastSpell command; alt-cost mutual exclusion; `cast_spell_aftermath` harness action.

**Resolved**: Embalm (CR 702.128) — validated 2026-03-01 (script stack/129, Sacred Cat, 12 unit tests in embalm.rs). Activated ability from graveyard (sorcery speed); exile card as cost (CR 602.2c); EmbalmAbility on stack; resolution creates token copy: white, no mana cost, Zombie in addition to original types, retains printed abilities; split second blocks activation; not a cast; KeywordAbility::Embalm disc 92 + AbilityDefinition::Embalm disc 25 + StackObjectKind::EmbalmAbility disc 27; Command::EmbalmCard; `embalm_card` harness action.

**Resolved**: Eternalize (CR 702.129) — validated 2026-03-01 (script stack/130, Proven Combatant, 12 unit tests in eternalize.rs). Activated ability from graveyard (sorcery speed); exile card as cost (CR 602.2c); EternalizeAbility on stack; resolution creates token copy: black, 4/4, no mana cost, Zombie in addition to original types, retains printed abilities; split second blocks activation; not a cast; KeywordAbility::Eternalize disc 93 + AbilityDefinition::Eternalize disc 26 + StackObjectKind::EternalizeAbility disc 28; Command::EternalizeCard; `eternalize_card` harness action.

**Resolved**: Encore (CR 702.141) — validated 2026-03-01 (script stack/131, Briarblade Adept, 10 unit tests in encore.rs). Activated ability from graveyard (sorcery speed); exile card as cost (CR 602.2c); EncoreAbility on stack; resolution creates one token copy per living opponent with Haste + encore_sacrifice_at_end_step + encore_must_attack tags; EncoreSacrificeTrigger at end step (sacrifice only if controller==activator); KeywordAbility::Encore disc 94 + AbilityDefinition::Encore disc 27 + StackObjectKind::EncoreAbility disc 29 + EncoreSacrificeTrigger disc 30; Command::EncoreCard; `encore_card` harness action; 3 new GameObject fields.

**Resolved**: Dash (CR 702.109) — validated 2026-03-01 (script stack/132, Zurgo Bellstriker, 7 unit tests in dash.rs). Alternative cost; haste + return-to-hand at end step; KeywordAbility::Dash disc 95 + AbilityDefinition::Dash disc 28 + StackObjectKind::DashReturnTrigger disc 31; CastSpell::cast_with_dash; casting.rs dash alt-cost validation + mutual exclusion; `cast_spell_dash` harness action.

**Resolved**: Blitz (CR 702.152) — validated 2026-03-02 (script stack/133, Riveteers Requisitioner, 9 unit tests in blitz.rs). Alternative cost; haste + "When this creature dies, draw a card" + delayed sacrifice at end step; KeywordAbility::Blitz disc 96 + AbilityDefinition::Blitz disc 29 + StackObjectKind::BlitzSacrificeTrigger disc 32 + PendingTriggerKind::BlitzSacrifice; AltCostKind::Blitz; casting.rs blitz alt-cost validation + mutual exclusion (12 other alt costs) + get_blitz_cost; resolution.rs ETB haste grant + draw-on-death trigger + BlitzSacrificeTrigger resolution; turn_actions.rs end-step sacrifice scan; StackObject::was_blitzed; `cast_spell_blitz` harness action.

**Resolved**: Plot (CR 702.170) — validated 2026-03-02 (script stack/134, Slickshot Show-Off, 20 unit tests in plot.rs). Special action from hand (CR 116.2k, does NOT use the stack); pay plot cost and exile card face-up; is_plotted + plotted_turn on GameObject; cast from exile on any later turn without paying mana cost (CR 702.170d); sorcery-speed timing for both plot action and free cast; KeywordAbility::Plot disc 97 + AbilityDefinition::Plot disc 30 + AltCostKind::Plot; Command::PlotCard + GameEvent::CardPlotted; rules/plot.rs special action handler; mutual exclusion with 14 other alt costs; `plot_card` + `cast_spell_plot` harness actions.

**Resolved**: Bargain (CR 702.166) — validated 2026-03-03 (script stack/137, Torch the Tower, 10 unit tests in bargain.rs). Optional additional cost (CR 118.8): sacrifice an artifact, enchantment, or token; KeywordAbility::Bargain disc 100 + Condition::WasBargained disc 10; CastSpell.bargain_sacrifice: Option<ObjectId>; casting.rs 6-check validation (keyword present, on battlefield, controlled by caster, artifact/enchantment/token) + sacrifice execution + was_bargained flag; was_bargained propagated StackObject -> GameObject -> EffectContext; copies of bargained spell also bargained (CR 707.2); cascade/suspend free casts excluded; `cast_spell_bargain` harness action.

**Resolved**: Spectacle (CR 702.137) — validated 2026-03-04 (script stack/139, Skewer the Critics, 11 unit tests in spectacle.rs). Alternative cost (CR 118.9a mutual exclusion with 15 other alt costs); `life_lost_this_turn` on PlayerState incremented at DealDamage/LoseLife/DrainLife/combat damage sites, reset at turn boundary, excluded for infect (CR 702.90b); KeywordAbility::Spectacle disc 102 + AbilityDefinition::Spectacle{cost} disc 34 + AltCostKind::Spectacle; casting.rs get_spectacle_cost + any_opponent_lost_life check (excludes eliminated/conceded players + caster self-damage); commander tax stacks on top (CR 118.9d); `cast_spell_spectacle` harness action.

**Resolved**: Surge (CR 702.117) — validated 2026-03-05 (script stack/140, Reckless Bushwhacker, 11 unit tests in surge.rs). Alternative cost (CR 118.9a mutual exclusion with 16 other alt costs); KeywordAbility::Surge disc 103 + AbilityDefinition::Surge{cost} disc 35 + AltCostKind::Surge; casting.rs cast_with_surge + get_surge_cost + spells_cast_this_turn >= 1 precondition; resolution.rs was_surged propagation to permanent; `cast_spell_surge` harness action.

**Resolved**: Casualty (CR 702.153) — validated 2026-03-05 (script stack/141, Make Disappear, 9 unit tests in casualty.rs). Optional additional cost (CR 118.8): sacrifice creature with power >= N to trigger a copy of the spell; KeywordAbility::Casualty(u32) disc 104 + StackObjectKind::CasualtyTrigger disc 34; CastSpell.casualty_sacrifice: Option<ObjectId>; casting.rs power-threshold validation + sacrifice execution + was_casualty_paid flag + CasualtyTrigger push; resolution.rs CasualtyTrigger resolves to create one copy; `cast_spell_casualty` harness action.

**Resolved**: Cumulative Upkeep (CR 702.24) — validated 2026-03-06 (script stack/152, Mystic Remora, 8 unit tests in cumulative_upkeep.rs). Triggered ability per CR 702.24a: upkeep adds age counter, then pay cost per age counter or sacrifice; CumulativeUpkeepCost enum supports Mana and Life costs; CumulativeUpkeepTrigger StackObjectKind; Command::PayCumulativeUpkeep; multiple instances trigger separately per CR 702.24b.

**Resolved**: Fading (CR 702.32) — validated 2026-03-06 (script stack/150, Blastoderm, 8 unit tests in fading.rs). Two abilities per CR 702.32a: (1) ETB replacement places N fade counters, (2) upkeep trigger removes a fade counter or sacrifices if unable; turn_actions.rs queues FadingUpkeep triggers (no intervening-if, unlike Vanishing); resolution.rs ETB hook + FadingTrigger resolution; lands.rs parallel ETB hook for land permanents; uses fade counters (not time counters); multiple Fading instances each trigger separately.

**Resolved**: Outlast (CR 702.107) — validated 2026-03-06 (script stack/158, Ainok Bond-Kin, 7 unit tests in outlast.rs). Convenience AbilityDefinition::Outlast{cost} expands into ActivatedAbility via enrich_spec_from_def: tap + mana cost, sorcery speed, +1/+1 counter on self; KeywordAbility::Outlast disc 121 + AbilityDefinition::Outlast disc 48; no custom StackObjectKind (generic ActivatedAbility path).

**Resolved**: Bloodthirst (CR 702.54) — validated 2026-03-06 (script stack/160, Stormblood Berserker, 8 unit tests in bloodthirst.rs). ETB replacement per CR 702.54a: places N +1/+1 counters if any opponent was dealt damage this turn; multiple instances add independently (CR 702.54c); KeywordAbility::Bloodthirst(u32) disc 123; `damage_dealt_this_turn` on PlayerState; resolution.rs ETB hook; `cast_creature` harness pattern.

**Resolved**: Champion (CR 702.72) — validated 2026-03-07 (script stack/164, Changeling Hero, 9 unit tests in champion.rs). ETB triggered ability per CR 702.72a: exile a creature you control matching the champion filter or sacrifice self; LTB triggered ability per CR 702.72b: return exiled card to battlefield under owner's control; CR 607.2 linked abilities; KeywordAbility::Champion disc 126 + ChampionFilter enum; AbilityDefinition::Champion disc 49; champion_exiled_card on GameObject; PendingTriggerKind::ChampionETB/LTB; StackObjectKind::ChampionETBTrigger disc 47 + ChampionLTBTrigger disc 48; LTB wired to CreatureDied/PermanentDestroyed/ObjectExiled/ObjectReturnedToHand events.
