# MTG Engine — Ability Coverage Audit

> Living document. Refresh with `/audit-abilities`.
> Last audited: 2026-02-25

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
| P1       | 42    | 29        | 3        | 10      | 0    | 0   |
| P2       | 17    | 0         | 0        | 0       | 17   | 0   |
| P3       | 40    | 0         | 0        | 0       | 40   | 0   |
| P4       | 100   | 0         | 0        | 0       | 88   | 12  |
| **Total**| **199**| **29**   | **3**    | **10**  | **145**| **12** |

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
| Intimidate | 702.13 | P1 | `partial` | `state/types.rs` (enum only) | — | — | — | In enum; no blocking restriction enforced |
| Fear | 702.36 | P3 | `none` | — | — | — | — | Can't be blocked except by artifact/black creatures |
| Shadow | 702.28 | P4 | `none` | — | — | — | — | Can only block/be blocked by shadow creatures |
| Horsemanship | 702.30 | P4 | `none` | — | — | — | — | Can only be blocked by horsemanship (Portal Three Kingdoms) |
| Skulk | 702.120 | P4 | `none` | — | — | — | — | Can't be blocked by creatures with greater power |
| Landwalk | 702.14 | P1 | `partial` | `state/types.rs` (enum only) | — | — | — | In enum; no unblockability rule enforced |
| CantBeBlocked | 509.1 | P1 | `complete` | `state/types.rs`, `rules/combat.rs` | Rogues Passage (pending) | — | — | Pseudo-keyword; blocking restriction enforced |

---

## Section 3: Equipment & Attachment

Keywords governing how permanents attach to other permanents.

| Ability | CR | Priority | Status | Engine File(s) | Card Def | Script | Depends On | Notes |
|---------|----|----------|--------|----------------|----------|--------|------------|-------|
| Equip | 702.6 | P1 | `partial` | `state/types.rs` (enum), `cards/definitions.rs` (static grants) | Lightning Greaves, Swiftfoot Boots | — | — | Static keyword-grant works; no `Command::EquipCreature` activation |
| Enchant | 702.5 | P1 | `partial` | `state/types.rs` (enum) | Enchantment cards | — | — | Aura attachment works; keyword itself not rule-enforced |
| Bestow | 702.103 | P3 | `none` | — | — | — | Enchant | Cast as Aura or creature; falls off → becomes creature |
| Reconfigure | 702.151 | P4 | `none` | — | — | — | Equip | Artifact creature that can attach/detach |
| Fortify | 702.67 | P4 | `none` | — | — | — | — | Equip for lands (Fortifications) |
| Living Weapon | 702.92 | P3 | `none` | — | — | — | Equip | ETB: create 0/0 Phyrexian Germ, attach |

---

## Section 4: Alternative Casting

Keywords that allow spells to be cast from non-hand zones or at alternate costs.

| Ability | CR | Priority | Status | Engine File(s) | Card Def | Script | Depends On | Notes |
|---------|----|----------|--------|----------------|----------|--------|------------|-------|
| Flashback | 702.34 | P2 | `none` | — | — | — | — | Cast from graveyard, then exile |
| Madness | 702.35 | P3 | `none` | — | — | — | — | When discarded, may cast for madness cost |
| Miracle | 702.94 | P3 | `none` | — | — | — | — | Reveal first drawn card, cast for miracle cost |
| Escape | 702.138 | P3 | `none` | — | — | — | — | Cast from graveyard by exiling other cards |
| Foretell | 702.143 | P3 | `none` | — | — | — | — | Exile face-down from hand, cast later for foretell cost |
| Retrace | 702.81 | P4 | `none` | — | — | — | — | Cast from graveyard by discarding a land |
| Jump-Start | 702.133 | P4 | `none` | — | — | — | — | Cast from graveyard by discarding a card |
| Aftermath | 702.127 | P4 | `none` | — | — | — | — | Cast second half from graveyard only |
| Disturb | 702.146 | P4 | `none` | — | — | — | — | Cast transformed from graveyard |
| Unearth | 702.84 | P3 | `none` | — | — | — | — | Return from graveyard to battlefield, exile at end step |
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
| Convoke | 702.51 | P2 | `none` | — | — | — | — | Tap creatures to pay mana costs |
| Delve | 702.66 | P2 | `none` | — | — | — | — | Exile cards from graveyard to pay generic mana |
| Improvise | 702.126 | P3 | `none` | — | — | — | — | Tap artifacts to pay generic mana |
| Affinity | 702.41 | P3 | `none` | — | — | — | — | Costs {1} less for each [type] you control |
| Undaunted | 702.124 | P3 | `none` | — | — | — | — | Costs {1} less for each opponent |
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
| Kicker | 702.33 | P2 | `none` | — | — | — | — | Optional additional cost for enhanced effect |
| Overload | 702.96 | P3 | `none` | — | — | — | — | Replace "target" with "each" |
| Replicate | 702.56 | P4 | `none` | — | — | — | — | Pay replicate cost N times → N copies |
| Splice | 702.47 | P4 | `none` | — | — | — | — | Reveal from hand, add text to another spell |
| Entwine | 702.42 | P4 | `none` | — | — | — | — | Pay entwine cost to choose all modes |
| Fuse | 702.102 | P4 | `none` | — | — | — | — | Cast both halves of a split card |
| Buyback | 702.27 | P3 | `none` | — | — | — | — | Pay buyback cost → return to hand on resolve |
| Spree | 702.165 | P4 | `none` | — | — | — | — | Choose modes, pay cost for each |
| Cleave | 702.148 | P4 | `none` | — | — | — | — | Pay cleave cost → remove bracketed text |
| Escalate | 702.121 | P4 | `none` | — | — | — | — | Pay escalate cost for each mode beyond the first |
| Split Second | 702.61 | P2 | `none` | — | — | — | — | No spells or activated abilities while on stack |
| Gravestorm | 702.69 | P4 | `none` | — | — | — | Storm | Copy for each permanent put into graveyard this turn |

---

## Section 7: Combat Triggers & Modifiers

Keywords that modify combat or trigger during combat.

| Ability | CR | Priority | Status | Engine File(s) | Card Def | Script | Depends On | Notes |
|---------|----|----------|--------|----------------|----------|--------|------------|-------|
| Flanking | 702.25 | P4 | `none` | — | — | — | — | Blocking creature without flanking gets -1/-1 |
| Bushido | 702.45 | P4 | `none` | — | — | — | — | +N/+N when blocks or becomes blocked |
| Provoke | 702.39 | P4 | `none` | — | — | — | — | Force target creature to block this |
| Exalted | 702.83 | P2 | `none` | — | — | — | — | +1/+1 when a creature attacks alone |
| Battle Cry | 702.91 | P3 | `none` | — | — | — | — | Attacking creatures get +1/+0 |
| Myriad | 702.116 | P3 | `none` | — | — | — | — | Create token copies attacking each other opponent |
| Melee | 702.122 | P4 | `none` | — | — | — | — | +1/+1 for each opponent attacked this combat |
| Enlist | 702.155 | P4 | `none` | — | — | — | — | Tap non-attacking creature to add its power |
| Annihilator | 702.86 | P2 | `none` | — | — | — | — | Defending player sacrifices N permanents |
| Dethrone | 702.105 | P3 | `none` | — | — | — | — | +1/+1 counter when attacking player with most life |
| Rampage | 702.23 | P4 | `none` | — | — | — | — | +N/+N for each creature blocking beyond first |
| Banding | 702.22 | P4 | `n/a` | — | — | — | — | Extremely complex, rarely used; intentionally deferred |
| Renown | 702.112 | P4 | `none` | — | — | — | — | Put +1/+1 counters on first combat damage to player |
| Afflict | 702.130 | P4 | `none` | — | — | — | — | Defending player loses N life when this is blocked |

---

## Section 8: Creature Enters/Leaves/Dies

Keywords triggered by creatures entering, leaving, or dying.

| Ability | CR | Priority | Status | Engine File(s) | Card Def | Script | Depends On | Notes |
|---------|----|----------|--------|----------------|----------|--------|------------|-------|
| Persist | 702.79 | P2 | `none` | — | — | — | — | Dies without -1/-1 counter → return with -1/-1 counter |
| Undying | 702.93 | P2 | `none` | — | — | — | — | Dies without +1/+1 counter → return with +1/+1 counter |
| Riot | 702.136 | P3 | `none` | — | — | — | — | ETB: choose haste or +1/+1 counter |
| Afterlife | 702.135 | P3 | `none` | — | — | — | — | Dies → create N 1/1 flying Spirit tokens |
| Exploit | 702.111 | P3 | `none` | — | — | — | — | ETB: may sacrifice a creature |
| Evoke | 702.74 | P2 | `none` | — | — | — | — | Alternative cost; sacrifice when ETB |
| Encore | 702.141 | P4 | `none` | — | — | — | — | Exile from graveyard → token copy for each opponent, attack, sacrifice at end |
| Champion | 702.72 | P4 | `none` | — | — | — | — | ETB exile a creature you control; leaves → return it |
| Devour | 702.82 | P4 | `none` | — | — | — | — | ETB: sacrifice creatures for +1/+1 counters |
| Tribute | 702.107 | P4 | `none` | — | — | — | — | Opponent chooses: +1/+1 counters or ability triggers |
| Fabricate | 702.123 | P4 | `none` | — | — | — | — | ETB: choose +1/+1 counters or create Servo tokens |
| Decayed | 702.145 | P4 | `none` | — | — | — | — | Can't block; sacrifice at end of combat when it attacks |
| Training | 702.150 | P4 | `none` | — | — | — | — | Attacks with greater-power creature → +1/+1 counter |
| Backup | 702.160 | P4 | `none` | — | — | — | — | ETB: put +1/+1 counters on target creature, it gains abilities |

---

## Section 9: Counters & Growth

Keywords involving counter manipulation and creature growth.

| Ability | CR | Priority | Status | Engine File(s) | Card Def | Script | Depends On | Notes |
|---------|----|----------|--------|----------------|----------|--------|------------|-------|
| Modular | 702.43 | P3 | `none` | — | — | — | — | ETB with +1/+1 counters; dies → move counters |
| Graft | 702.58 | P4 | `none` | — | — | — | — | ETB with +1/+1 counters; move to entering creatures |
| Evolve | 702.100 | P3 | `none` | — | — | — | — | Creature enters with greater P or T → +1/+1 counter |
| Scavenge | 702.97 | P4 | `none` | — | — | — | — | Exile from graveyard → put +1/+1 counters on creature |
| Outlast | 702.107 | P4 | `none` | — | — | — | — | Tap + mana → +1/+1 counter (sorcery speed) |
| Amplify | 702.38 | P4 | `none` | — | — | — | — | Reveal creature cards from hand for +1/+1 counters |
| Adapt | — | P3 | `none` | — | — | — | — | If no +1/+1 counters → put N +1/+1 counters (ability word, not keyword) |
| Bolster | — | P3 | `none` | — | — | — | — | Put +1/+1 counters on creature with least toughness (keyword action, not keyword) |

---

## Section 10: Upkeep, Time & Phasing

Keywords involving time-based effects, phasing, and recurring costs.

| Ability | CR | Priority | Status | Engine File(s) | Card Def | Script | Depends On | Notes |
|---------|----|----------|--------|----------------|----------|--------|------------|-------|
| Cycling | 702.29 | P2 | `none` | — | — | — | — | Discard + pay cost → draw a card |
| Suspend | 702.62 | P3 | `none` | — | — | — | — | Exile with time counters; remove each upkeep; cast when last removed |
| Phasing | 702.26 | P4 | `none` | — | — | — | — | Phases out/in on untap step; deferred (corner case audit) |
| Cumulative Upkeep | 702.24 | P4 | `none` | — | — | — | — | Increasing cost each upkeep |
| Echo | 702.31 | P4 | `none` | — | — | — | — | Pay mana cost again on next upkeep or sacrifice |
| Fading | 702.32 | P4 | `none` | — | — | — | — | ETB with fade counters; remove each upkeep; sacrifice at 0 |
| Vanishing | 702.63 | P4 | `none` | — | — | — | — | ETB with time counters; remove each upkeep; sacrifice at 0 |
| Forecast | 702.57 | P4 | `none` | — | — | — | — | Reveal from hand during upkeep for effect |
| Recover | 702.59 | P4 | `none` | — | — | — | — | When a creature dies, return this from graveyard |
| Dredge | 702.52 | P2 | `none` | — | — | — | — | Replace draw with mill N + return from graveyard |

---

## Section 11: Commander & Multiplayer

Keywords specifically relevant to Commander or multiplayer.

| Ability | CR | Priority | Status | Engine File(s) | Card Def | Script | Depends On | Notes |
|---------|----|----------|--------|----------------|----------|--------|------------|-------|
| Partner | 702.124 | P1 | `validated` | `rules/commander.rs` | — | `commander/` scripts | — | Two commanders; deck validation enforced |
| Companion | 702.139 | P1 | `validated` | `rules/commander.rs`, `rules/engine.rs` | — | `commander/` scripts | — | Start in sideboard; bring to hand for {3} |
| Partner With | 702.124 | P3 | `none` | — | — | — | Partner | Specific partner pairs; search on ETB |
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
| Changeling | 702.73 | P2 | `none` | — | — | — | — | Has all creature types |
| Crew | 702.122 | P2 | `none` | — | — | — | — | Tap creatures with total power >= N to animate Vehicle |
| Saddle | 702.163 | P4 | `none` | — | — | — | Crew | Crew variant for Mounts |
| Prototype | 702.157 | P4 | `none` | — | — | — | — | Alternative smaller casting |
| Living Metal | — | P4 | `none` | — | — | — | — | Artifact is also a creature on your turn |
| Totem Armor | 702.89 | P4 | `none` | — | — | — | Enchant | Aura destroyed instead of enchanted permanent |
| Soulbond | 702.95 | P4 | `none` | — | — | — | — | Pair with another creature for shared abilities |
| Haunt | 702.55 | P4 | `none` | — | — | — | — | When this dies, exile haunting a creature |
| Extort | 702.101 | P3 | `none` | — | — | — | — | Pay W/B when casting → drain 1 from each opponent |
| Cipher | 702.99 | P4 | `none` | — | — | — | — | Encode spell on creature; cast copy on combat damage |
| Bloodthirst | 702.54 | P4 | `none` | — | — | — | — | ETB with +1/+1 counters if opponent was dealt damage |
| Bloodrush | — | P4 | `none` | — | — | — | — | Ability word; discard to pump attacking creature |
| Devoid | 702.114 | P4 | `none` | — | — | — | — | Colorless regardless of mana cost |
| Ingest | 702.115 | P4 | `none` | — | — | — | — | Combat damage to player → exile top card of library |
| Wither | 702.80 | P3 | `none` | — | — | — | — | Damage dealt as -1/-1 counters |
| Infect | 702.90 | P3 | `none` | — | — | — | — | Damage to creatures as -1/-1 counters, to players as poison |
| Poisonous | 702.70 | P4 | `none` | — | — | — | — | Combat damage to player → poison counters |
| Toxic | 702.156 | P4 | `none` | — | — | — | — | Combat damage to player → poison counters (fixed number) |
| Corrupted | — | P4 | `none` | — | — | — | — | Ability word; if opponent has 3+ poison counters |
| Hideaway | 702.75 | P3 | `none` | — | — | — | — | ETB: look at top N, exile one face-down; cast when condition met |
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
| Connive | 702.153 | P3 | `none` | — | — | — | — | Draw, discard; if nonland discarded → +1/+1 counter |
| Casualty | 702.154 | P4 | `none` | — | — | — | — | Sacrifice creature with power >= N → copy spell |
| Alliance | — | P4 | `none` | — | — | — | — | Ability word; trigger when creature ETBs under your control |
| Ravenous | — | P4 | `none` | — | — | — | — | ETB with X +1/+1 counters; draw if X >= 5 |
| Squad | 702.159 | P4 | `none` | — | — | — | — | Pay squad cost N times → N token copies on ETB |
| Enrage | — | P4 | `none` | — | — | — | — | Ability word; trigger when dealt damage |
| Ascend | 702.131 | P3 | `none` | — | — | — | — | City's blessing if 10+ permanents |
| Treasure tokens | — | P2 | `none` | — | — | — | — | Predefined token: sacrifice → add one mana of any color |
| Food tokens | — | P3 | `none` | — | — | — | — | Predefined token: {2}, tap, sacrifice → gain 3 life |
| Clue tokens | — | P3 | `none` | — | — | — | — | Predefined token: {2}, sacrifice → draw a card |
| Blood tokens | — | P4 | `none` | — | — | — | — | Predefined token: {1}, tap, discard, sacrifice → draw |
| Prowess | 702.108 | P1 | `validated` | `state/types.rs`, `state/hash.rs`, `state/game_object.rs`, `state/builder.rs`, `state/continuous_effect.rs`, `rules/abilities.rs`, `effects/mod.rs` | Monastery Swiftspear | `stack/056` | — | KeywordAbility::Prowess enum + TriggerEvent::ControllerCastsNoncreatureSpell dispatch; EffectFilter::Source in continuous_effect.rs; TriggeredAbilityDef auto-expansion in builder.rs; 8 unit tests in `tests/prowess.rs`; game script 056 (pending_review, all assertions pass) |
| Regenerate | 701.15 | P3 | `none` | — | — | — | — | Keyword action (not ability): replace destruction with tap+remove from combat |
| Proliferate | 701.27 | P3 | `none` | — | — | — | — | Keyword action: add counter to any permanent/player with counters |
| Transform | 701.28 | P3 | `none` | — | — | — | — | Keyword action: flip DFC to other face |
| Daybound/Nightbound | 702.145 | P4 | `none` | — | — | — | Transform | DFC auto-transform based on day/night cycle |
| Investigate | 701.36 | P3 | `none` | — | — | — | Clue tokens | Keyword action: create a Clue token |
| Amass | 701.44 | P4 | `none` | — | — | — | — | Put +1/+1 counters on Army token or create one |
| Discover | 702.161 | P4 | `none` | — | — | — | Cascade | Cascade variant without the free cast restriction |
| Forage | 701.55 | P4 | `none` | — | — | — | — | Sacrifice a Food or exile 3 cards from graveyard |
| Offspring | 702.167 | P4 | `none` | — | — | — | — | Pay offspring cost → create 1/1 token copy on ETB |
| Impending | 702.168 | P4 | `none` | — | — | — | — | Cast for less as non-creature with time counters |
| Gift | 702.169 | P4 | `none` | — | — | — | — | Choose an opponent to receive a gift |
| Collect evidence | 701.53 | P4 | `none` | — | — | — | — | Exile cards from graveyard with total MV >= N |
| Suspect | 701.52 | P4 | `none` | — | — | — | — | Menace + can't block |
| Surveil | 701.42 | P2 | `none` | — | — | — | — | Look at top N, put in graveyard or on top |
| Adapt (keyword action) | — | P3 | `none` | — | — | — | — | See Section 9 |
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
| Dies trigger | P1 | `partial` | `cards/card_definition.rs:468` | Solemn Simulacrum | — | — | `WhenDies` TriggerCondition defined; runtime `check_triggers` incomplete |
| Attack trigger | P1 | `partial` | `cards/card_definition.rs:470` | — | — | — | `WhenAttacks` TriggerCondition defined; no runtime dispatch |
| Combat damage trigger | P1 | `partial` | `cards/card_definition.rs:474` | Alela (approximated) | — | — | `WhenDealsCombatDamageToPlayer` defined; dispatch incomplete |
| Opponent-casts trigger | P1 | `partial` | `cards/card_definition.rs:476` | Rhystic Study | — | — | `WheneverOpponentCastsSpell` defined; trigger dispatch partial |
| Search library | P1 | `complete` | `effects/` | Wayfarer's Bauble, Evolving Wilds, Terramorphic Expanse, Cultivate | — | — | `SearchLibrary` effect works; harness doesn't emit player command yet |
| Destroy + compensate | P1 | `validated` | `effects/` | Beast Within, Generous Gift, Pongify, Rapid Hybridization | `baseline/` scripts | — | Destroy target + create token for controller |
| Mass removal | P1 | `validated` | `effects/` | Wrath of God, Damnation, Supreme Verdict, Blasphemous Act | `baseline/` scripts | — | Destroy/damage all creatures |
| Counter spell | P1 | `validated` | `effects/` | Counterspell, Negate, Swan Song, Arcane Denial | `stack/` scripts | — | Counter target spell on stack |
| Global replacement | P1 | `validated` | `effects/`, `rules/replacement.rs` | Rest in Peace, Leyline of the Void | `replacement/` scripts | — | Replace zone change events globally |
| Equipment keyword grant | P1 | `validated` | `cards/definitions.rs`, `rules/layers.rs` | Lightning Greaves, Swiftfoot Boots | `layers/` scripts | — | Layer 6 continuous effect granting keywords |
| Modal choice | P2 | `none` | — | — | — | — | "Choose one —" modal spells not yet supported |
| Declare attackers action | P1 | `partial` | `rules/combat.rs` | — | — | — | Combat declaration works; harness action for scripts incomplete |
| Declare blockers action | P1 | `partial` | `rules/combat.rs` | — | — | — | Combat declaration works; harness action for scripts incomplete |

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
| Evasion / blocking restriction | `rules/combat.rs` | Flying, Menace, Intimidate |
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

1. **Equip activation** — Static keyword grant works, but no `Command::EquipCreature` exists. Equipment can't be moved between creatures via player action. Blocks combat scripts involving equipment.
2. **Intimidate blocking rule** — In enum but no combat restriction enforced.
3. **Landwalk unblockability** — In enum but no blocking check.
4. **Dies trigger dispatch** — `WhenDies` condition defined but `check_triggers` doesn't fire it.
5. **Attack trigger dispatch** — `WhenAttacks` condition defined but no runtime dispatch.
6. **Combat damage trigger dispatch** — `WhenDealsCombatDamageToPlayer` defined but dispatch incomplete.
7. **Opponent-casts trigger dispatch** — `WheneverOpponentCastsSpell` defined but partial.
8. **Declare attackers/blockers harness action** — Combat works but scripts can't issue the action.

### P2 Gaps (Commander staples)

1. **Flashback** (CR 702.34) — Top Commander mechanic; no implementation.
2. **Cycling** (CR 702.29) — Common Commander draw smoothing; no implementation.
3. **Dredge** (CR 702.52) — Powerful graveyard mechanic; no implementation.
4. **Convoke** (CR 702.51) — Token decks rely on this; no implementation.
5. **Delve** (CR 702.66) — Graveyard cost reduction; no implementation.
6. **Kicker** (CR 702.33) — Very common optional cost; no implementation.
7. **Split Second** (CR 702.61) — Priority restriction; no implementation.
8. **Changeling** (CR 702.73) — Tribal synergy staple; no implementation.
9. **Crew** (CR 702.122) — Vehicle animation; no implementation.
10. **Exalted** (CR 702.83) — Voltron staple; no implementation.
