# Ability Batch Implementation Plan

> **Purpose**: Implement ALL keyword abilities and ability patterns before advancing to M10.
> Organizes ~75 implementable abilities into batches grouped by engine infrastructure,
> minimizing context-switching and maximizing code reuse within each batch.
>
> **Companion doc**: `docs/workstream-coordination.md` (cross-workstream coordination)
>
> **Last updated**: 2026-02-28

---

## Scope

| Category | Count | Notes |
|----------|-------|-------|
| Remaining P3 | 5 | Overload, Bolster, Adapt, Partner With, Modal choice (P2) |
| Implementable P4 | ~70 | Full list below |
| Phasing (unblocked) | 1 | Moved to Batch 8 — `phased_out` field already exists |
| Mutate (mini-milestone) | 1 | Dedicated section after main batches |
| Blocked (needs new subsystem) | 9 | Morph tree (5), Transform tree (4) |
| Excluded (n/a) | 12 | Digital-only, Banding, Venture/Dungeon, Ring Tempts |
| Hideaway (WIP) | 1 | In review — finish first |
| **Total to implement** | **~78** | Including Phasing + Mutate |

### Estimated pace

Each ability follows the pipeline: enum variant → enforcement → triggers → tests.
Simple abilities (reusing existing infra): ~20-40 min each.
Medium abilities (small new patterns): ~1-2 hours each.
Complex abilities (new commands or subsystems): ~2-4 hours each.

Batches are sized at 5-8 abilities. Target: **1-2 batches per session** for simple
batches, **1 batch per session** for complex ones. ~12-15 sessions total.

---

## Blocked Abilities (implement later when subsystems are built)

### Morph/Face-Down Subsystem (5 abilities)

Needs: face-down casting path, characteristic suppression in layers, turn-face-up
special action, face-down battlefield state management.

| Ability | CR | Depends on |
|---------|-----|-----------|
| Morph | 702.37 | Face-down subsystem |
| Megamorph | 702.37 | Morph + counter |
| Disguise | 702.162 | Morph + Ward {2} |
| Manifest | 701.34 | Face-down subsystem |
| Cloak | 701.56 | Manifest + Ward {2} |

### Transform/DFC Subsystem (4 abilities)

Needs: second face on CardDefinition, `transformed: bool` on GameObject,
Transform keyword action, day/night global state.

| Ability | CR | Depends on |
|---------|-----|-----------|
| Transform | 701.28 | DFC second face |
| Disturb | 702.146 | Transform |
| Daybound/Nightbound | 702.145 | Transform + day/night cycle |
| Craft | 702.158 | Transform |

**Total blocked: 9 abilities. Defer to a dedicated "face-down & DFC" milestone.**

> **Note**: Phasing was moved to Batch 8 (implementable — `phased_out` field already exists).
> Mutate has its own mini-milestone after the main batches (see below).

---

## Implementation Batches

### Batch 0: Finish WIP + P3 Stragglers (1 session)

Clear existing pipeline state before starting P4 work.

| # | Ability | Priority | CR | Effort | Notes |
|---|---------|----------|-----|--------|-------|
| 0.1 | Hideaway | P3 | 702.75 | Done | Finish steps 5-7 (card, script, coverage) |
| 0.2 | Overload | P3 | 702.96 | Medium | "Target" → "each" text replacement |
| 0.3 | Bolster | P3 | — | Low | Put +1/+1 on creature with least toughness |
| 0.4 | Adapt | P3 | — | Low | If no +1/+1 counters → put N counters |
| 0.5 | Partner With | P3 | 702.124 | Medium | ETB search for specific partner |

- [x] Batch 0 complete

---

### Batch 1: Evasion & Simple Keywords (6 abilities, ~1 session)

All follow the same pattern as Flying/Fear/Intimidate in `combat.rs`: add enum
variant, add blocking restriction check, add test.

| # | Ability | CR | Effort | Pattern |
|---|---------|-----|--------|---------|
| 1.1 | Shadow | 702.28 | Low | Block restriction: only shadow ↔ shadow |
| 1.2 | Horsemanship | 702.30 | Low | Block restriction: only horsemanship ↔ horsemanship |
| 1.3 | Skulk | 702.120 | Low | Block restriction: blocker power > this power |
| 1.4 | Devoid | 702.114 | Low | CDA: colorless regardless of mana cost (Layer 5) |
| 1.5 | Decayed | 702.145 | Low | Can't block + sacrifice at end of combat (Myriad pattern) |
| 1.6 | Ingest | 702.115 | Low | Combat damage trigger → exile top of opponent's library |

**Infrastructure reused**: `combat.rs` blocking checks, `EndOfCombat` trigger (Myriad),
combat damage trigger (existing).

- [x] Batch 1 complete

---

### Batch 2: Combat Triggers — Blocking (7 abilities, ~1-2 sessions)

Abilities that trigger when creatures block or are blocked. All wire into the
declare-blockers step in `combat.rs`.

| # | Ability | CR | Effort | Pattern |
|---|---------|-----|--------|---------|
| 2.1 | Flanking | 702.25 | Low | When blocked by non-flanker → -1/-1 |
| 2.2 | Bushido | 702.45 | Low | When blocks or is blocked → +N/+N until EOT |
| 2.3 | Rampage | 702.23 | Low | +N/+N for each blocker beyond first |
| 2.4 | Provoke | 702.39 | Medium | Force target creature to block this |
| 2.5 | Afflict | 702.130 | Low | When blocked → defending player loses N life |
| 2.6 | Renown | 702.112 | Low | First combat damage to player → +1/+1 counters + renowned flag |
| 2.7 | Training | 702.150 | Low | Attacks with greater-power creature → +1/+1 counter |

**Infrastructure reused**: `CombatState::blockers_for()`, declare-blocker triggers,
combat damage triggers. Renown needs a `is_renowned: bool` on GameObject.

- [x] Batch 2 complete

---

### Batch 3: Combat Modifiers & Ninjutsu (5 abilities, ~1-2 sessions)

More combat mechanics, including the medium-effort Ninjutsu.

| # | Ability | CR | Effort | Pattern |
|---|---------|-----|--------|---------|
| 3.1 | Melee | 702.122 | Low | +1/+1 for each opponent attacked this combat |
| 3.2 | Enlist | 702.155 | Medium | Tap non-attacking creature → add its power |
| 3.3 | Poisonous | 702.70 | Low | Combat damage → N poison counters (Infect infra) |
| 3.4 | Toxic | 702.156 | Low | Combat damage → fixed N poison counters (Infect infra) |
| 3.5 | Ninjutsu | 702.49 | Medium | Swap unblocked attacker for creature in hand; new Command |

**Infrastructure reused**: `CombatState::is_blocked()`, poison counter SBA (Infect),
`MoveZone` to hand. Ninjutsu needs `Command::ActivateNinjutsu`.

- [x] Batch 3 complete

---

### Batch 4: Alternate Casting from Graveyard (6 abilities, ~1-2 sessions)

All follow the Flashback/Unearth pattern: cast or activate from graveyard with
a special cost, with zone-redirect on resolution.

| # | Ability | CR | Effort | Pattern |
|---|---------|-----|--------|---------|
| 4.1 | Retrace | 702.81 | Low | Flashback variant: discard a land as additional cost |
| 4.2 | Jump-Start | 702.133 | Low | Flashback variant: discard a card as additional cost |
| 4.3 | Aftermath | 702.127 | Medium | Cast second half of split card from graveyard only |
| 4.4 | Embalm | 702.128 | Medium | Exile from graveyard → create white token copy (no mana cost) |
| 4.5 | Eternalize | 702.129 | Medium | Exile from graveyard → create 4/4 black token copy |
| 4.6 | Encore | 702.141 | Medium | Exile from graveyard → token copy per opponent, haste, attack, sacrifice EOT |

**Infrastructure reused**: `casting.rs` alternate-cost machinery, `CreateToken`,
graveyard zone activation (Unearth pattern), end-of-turn sacrifice (Unearth pattern).

- [x] Batch 4 complete

---

### Batch 5: Alternate Casting from Hand/Exile (5 abilities, ~1 session)

Alternate cost patterns for hand-cast spells, all reusing `casting.rs` machinery.

| # | Ability | CR | Effort | Pattern |
|---|---------|-----|--------|---------|
| 5.1 | Dash | 702.109 | Low | Alt cost, haste, return to hand at EOT (Unearth pattern) |
| 5.2 | Blitz | 702.152 | Low | Alt cost, haste, draw-on-death trigger, sacrifice EOT |
| 5.3 | Plot | 702.164 | Low | Exile from hand (like Foretell), cast free later |
| 5.4 | Prototype | 702.157 | Medium | Alt smaller casting with different P/T and color |
| 5.5 | Impending | 702.168 | Medium | Cast for less as non-creature with time counters → becomes creature when counters removed |

**Infrastructure reused**: Foretell exile-then-cast, Unearth end-of-turn sacrifice,
Suspend time-counter removal.

- [x] Batch 5 complete

---

### Batch 6: Cost Modification (6 abilities, ~1-2 sessions)

Additional or alternative costs that modify how spells are cast.

| # | Ability | CR | Effort | Pattern |
|---|---------|-----|--------|---------|
| 6.1 | Bargain | 702.166 | Low | Additional cost: sacrifice artifact/enchantment/token |
| 6.2 | Emerge | 702.119 | Medium | Alt cost: sacrifice creature, reduce by its MV |
| 6.3 | Spectacle | 702.137 | Low | Alt cost if opponent lost life this turn |
| 6.4 | Surge | 702.117 | Low | Alt cost if you/teammate cast a spell this turn |
| 6.5 | Casualty | 702.154 | Medium | Additional cost: sacrifice creature with power >= N → copy spell |
| 6.6 | Assist | 702.132 | Medium | Another player may pay generic mana portion |

**Infrastructure reused**: Convoke/Improvise cost reduction, sacrifice-as-cost,
`spells_cast_this_turn` counter (Storm), spell copy (Storm/Cascade).

- [x] Batch 6 complete

---

### Batch 7: Spell Modifiers & Copies (7 abilities, ~2 sessions)

Mechanics that modify spells on the stack — copies, modes, text changes.

| # | Ability | CR | Effort | Pattern |
|---|---------|-----|--------|---------|
| 7.1 | Replicate | 702.56 | Medium | Pay N times → N copies on stack (Storm variant) |
| 7.2 | Gravestorm | 702.69 | Low | Storm variant: count permanents to graveyard |
| 7.3 | Overload (P3) | 702.96 | Medium | Alt cost: replace "target" with "each" |
| 7.4 | Cleave | 702.148 | Medium | Alt cost: remove bracketed text |
| 7.5 | Splice | 702.47 | High | Reveal from hand, add text to Arcane spell |
| 7.6 | Entwine | 702.42 | Medium | Pay to choose all modes (needs Modal choice) |
| 7.7 | Escalate | 702.121 | Medium | Pay per additional mode (needs Modal choice) |

**Note**: Entwine and Escalate depend on Modal choice (Batch 11). Implement those
two after Batch 11, or stub with auto-all-modes for now.

**Infrastructure reused**: Storm copy machinery (`copy.rs`), casting alt-cost framework.

- [x] Batch 7 complete

---

### Batch 8: Upkeep, Time & Phasing Mechanics (7 abilities, ~2-3 sessions)

Abilities triggered during upkeep, all following the Suspend pattern of
counter-removal + sacrifice/effect. Plus Phasing, which also happens at untap step.

| # | Ability | CR | Effort | Pattern |
|---|---------|-----|--------|---------|
| 8.1 | Vanishing | 702.63 | Low | ETB with time counters; remove each upkeep; sacrifice at 0 (= Suspend) |
| 8.2 | Fading | 702.32 | Low | ETB with fade counters; remove each upkeep; sacrifice at 0 |
| 8.3 | Echo | 702.31 | Low | Pay mana cost again next upkeep or sacrifice |
| 8.4 | Cumulative Upkeep | 702.24 | Medium | Age counter + increasing cost each upkeep |
| 8.5 | Recover | 702.59 | Low | When a creature dies → return this from graveyard (pay or exile) |
| 8.6 | Forecast | 702.57 | Medium | Reveal from hand during upkeep for effect |
| 8.7 | **Phasing** | 702.26 | **Medium** | Untap step: phase out/in; filter phased-out from battlefield checks |

**Phasing approach**: `phased_out: bool` already exists on `ObjectStatus` (`game_object.rs:273`),
already hashed (`hash.rs:529`), initialized to `false` in builder. Implementation:
1. Add `Phasing` variant to `KeywordAbility` enum + all match arms
2. Untap step (`turn_actions.rs`): phase in all phased-out permanents, phase out permanents with Phasing
3. Filter phased-out objects from battlefield queries across ~8 rules files (combat, targeting, SBAs, layers, etc.)
4. Indirect phasing: when a permanent phases out, Auras/Equipment attached to it also phase out (CR 702.26d)
5. Tests: phase-out skips untap, phased-out creature can't block/attack/be targeted, indirect phasing for Auras

**Infrastructure reused**: Suspend time-counter infrastructure, upkeep triggers,
sacrifice SBA. Phasing uses existing `phased_out` field.

- [x] Batch 8 complete

---

### Batch 9: Counter & Growth Mechanics (6 abilities, ~1 session)

Counter placement and movement patterns, all reusing AddCounter/RemoveCounter effects.

| # | Ability | CR | Effort | Pattern |
|---|---------|-----|--------|---------|
| 9.1 | Graft | 702.58 | Medium | ETB with counters; move counter to entering creatures |
| 9.2 | Scavenge | 702.97 | Low | Activated from graveyard → put counters on creature |
| 9.3 | Outlast | 702.107 | Low | Activated: tap + mana → +1/+1 counter, sorcery speed |
| 9.4 | Amplify | 702.38 | Medium | Reveal creatures from hand for +1/+1 counters on ETB |
| 9.5 | Bloodthirst | 702.54 | Low | ETB counters if opponent was dealt damage this turn |
| 9.6 | Amass | 701.44 | Medium | Put counters on Army or create Army token |

**Infrastructure reused**: `AddCounter`, `RemoveCounter`, `CreateToken`, ETB triggers,
graveyard activation (Unearth).

- [ ] Batch 9 complete

---

### Batch 10: ETB/Dies Patterns (7 abilities, ~2 sessions)

Creatures with enter-the-battlefield or leaves-the-battlefield effects.

| # | Ability | CR | Effort | Pattern |
|---|---------|-----|--------|---------|
| 10.1 | Devour | 702.82 | Low | ETB: sacrifice creatures → +1/+1 counters |
| 10.2 | Backup | 702.160 | Medium | ETB: counters on target + grant abilities |
| 10.3 | Champion | 702.72 | Medium | ETB exile own creature; LTB return it |
| 10.4 | Totem Armor | 702.89 | Low | Replacement: destroy aura instead of enchanted creature |
| 10.5 | Living Metal | — | Low | Artifact is creature on your turn (continuous effect) |
| 10.6 | Soulbond | 702.95 | Medium | Pair with creature; shared abilities while paired |
| 10.7 | Fortify | 702.67 | Low | Equip for lands (Equip variant) |

**Infrastructure reused**: ETB triggers, replacement effects (`replacement.rs`),
continuous effects (`layers.rs`), equip machinery.

- [ ] Batch 10 complete

---

### Batch 11: Modal Choice + Dependents (5 abilities, ~2 sessions)

Implement the Modal choice system (P2 gap), then abilities that depend on it.

| # | Ability | CR | Effort | Pattern |
|---|---------|-----|--------|---------|
| 11.1 | **Modal Choice** | 700.2 | High | "Choose one —" system; auto-first for bots, player choice later |
| 11.2 | Tribute | 702.107 | Medium | Opponent chooses: counters or ability triggers |
| 11.3 | Fabricate | 702.123 | Medium | Choose: +1/+1 counters or Servo tokens |
| 11.4 | Fuse | 702.102 | Medium | Cast both halves of split card |
| 11.5 | Spree | 702.165 | Medium | Choose modes, pay cost for each |

**Note**: Entwine (7.6) and Escalate (7.7) can be completed after this batch.

- [ ] Batch 11 complete

---

### Batch 12: Ability Words (trigger patterns) (5 abilities, ~1 session)

Ability words aren't keywords — they're just naming conventions for trigger patterns.
Each is a TriggerCondition + effect.

| # | Ability | CR | Effort | Pattern |
|---|---------|-----|--------|---------|
| 12.1 | Enrage | — | Low | Trigger: when this creature is dealt damage |
| 12.2 | Alliance | — | Low | Trigger: whenever a creature ETBs under your control |
| 12.3 | Corrupted | — | Low | Condition: if opponent has 3+ poison counters |
| 12.4 | Ravenous | — | Low | ETB with X +1/+1 counters; draw if X >= 5 |
| 12.5 | Bloodrush | — | Low | Discard to pump attacking creature |

**Infrastructure reused**: Existing trigger conditions, ETB counters, discard-as-cost.

- [ ] Batch 12 complete

---

### Batch 13: Newer Set Mechanics (8 abilities, ~2 sessions)

Recent set mechanics (2022-2024 releases).

| # | Ability | CR | Effort | Pattern |
|---|---------|-----|--------|---------|
| 13.1 | Discover | 702.161 | Low | Cascade variant (cascade infra exists) |
| 13.2 | Suspect | 701.52 | Low | Menace + can't block (keyword grant) |
| 13.3 | Collect Evidence | 701.53 | Low | Exile cards from graveyard with total MV >= N |
| 13.4 | Forage | 701.55 | Low | Sacrifice Food or exile 3 from graveyard |
| 13.5 | Squad | 702.159 | Medium | Pay N times → N token copies on ETB |
| 13.6 | Offspring | 702.167 | Medium | Pay offspring cost → 1/1 token copy on ETB |
| 13.7 | Gift | 702.169 | Medium | Choose opponent to receive a gift (bonus) |
| 13.8 | Saddle | 702.163 | Low | Crew variant for Mounts |

**Infrastructure reused**: Cascade (`copy.rs`), `CreateToken`, Crew, sacrifice-as-cost.

- [ ] Batch 13 complete

---

### Batch 14: Niche Keywords & Encoding (6 abilities, ~2 sessions)

The most exotic mechanics — some need small new tracking state.

| # | Ability | CR | Effort | Pattern |
|---|---------|-----|--------|---------|
| 14.1 | Cipher | 702.99 | Medium | Encode spell on creature; copy on combat damage to player |
| 14.2 | Haunt | 702.55 | Medium | Dies → exile haunting creature; trigger on that creature's death |
| 14.3 | Reconfigure | 702.151 | Medium | Artifact creature attaches/detaches as equipment |
| 14.4 | Blood Tokens | — | Low | Predefined token: {1}, T, discard → draw (like Clue/Food) |
| 14.5 | Treasure Tokens | — | Low | Predefined token: sacrifice → add one mana (if not already done) |
| 14.6 | Decayed tokens | — | Low | Token modifier: can't block, sacrifice after attacking |

- [ ] Batch 14 complete

---

### Batch 15: Commander Variants (3 abilities, ~30 min)

These are all Partner variants — structural deck-validation changes in `commander.rs`.

| # | Ability | CR | Effort | Pattern |
|---|---------|-----|--------|---------|
| 15.1 | Friends Forever | 702.124 | Low | Partner variant (Stranger Things) |
| 15.2 | Choose a Background | 702.124 | Low | Partner variant for Background enchantments |
| 15.3 | Doctor's Companion | 702.124 | Low | Partner variant (Doctor Who) |

**Infrastructure reused**: Partner rules in `commander.rs`.

- [ ] Batch 15 complete

---

### Mutate Mini-Milestone (1 ability, ~2-3 sessions)

Mutate (CR 702.140) requires a new merged-permanent model that doesn't fit into any
batch. Implement as a dedicated mini-milestone after all main batches are complete.

**Why it's separate**: Mutate needs `merged_cards: Vec<ObjectId>` on `GameObject` — a
structural change to the core object model. Normal abilities add behavior; Mutate changes
what a permanent *is*.

| # | Task | Effort | Description |
|---|------|--------|-------------|
| M.1 | Data model | Medium | Add `merged_cards: Vec<ObjectId>` to `GameObject`; hash it; `top_card()` accessor |
| M.2 | Cast command | Medium | `Command::CastWithMutate { target }` — cast creature targeting non-Human you control |
| M.3 | Resolution merge | High | On resolve: caster chooses over/under; merged permanent has top card's characteristics + ALL abilities from all cards |
| M.4 | Zone-change split | Medium | When merged permanent changes zones, all cards move together; if it dies, each card is a separate card in graveyard (CR 729.5) |
| M.5 | Mutate trigger | Low | "Whenever this creature mutates" trigger fires on successful merge |
| M.6 | Tests | Medium | Basic mutate, over/under choice, zone-change splitting, mutate trigger, interaction with Auras/Equipment |
| M.7 | Card definitions | Low | Gemrazer, Nethroi, Brokkos (popular Ikoria commanders) |

**Minimal viable scope**: Top card characteristics + all abilities + zone-change splits.
Covers the 4-5 popular Ikoria commanders that show up in actual Commander games.

**Deferred complexity**: Mutate with copy effects (CR 729.8), mutate with face-down
creatures (CR 729.6), mutate token ownership — handle if needed.

- [ ] Mutate mini-milestone complete

---

## Batch Dependencies

```
Batch 0 (P3 stragglers)
    │
    ▼
Batches 1-6, 8-10, 12-15 ── all independent, any order
    │                           (Batch 8 now includes Phasing)
    │   Batch 11 (Modal Choice)
    │       │
    │       ▼
    │   Batch 7.6, 7.7 (Entwine, Escalate)
    │
    ▼
All batches complete
    │
    ▼
Mutate Mini-Milestone (~2-3 sessions)
    │
    ▼
Blocked subsystems (optional dedicated milestone):
    ├── Face-Down/Morph (5 abilities)
    └── Transform/DFC (4 abilities)
```

Most batches are independent. The only hard dependency is:
- **Batch 11 (Modal Choice)** before Entwine (7.6), Escalate (7.7), Tribute (11.2), Fabricate (11.3)
- **All batches** before Mutate Mini-Milestone (needs stable object model)

---

## Card Authoring Requirements

Each batch needs 1-2 showcase cards per ability for validation. Estimate:

| Batch | Cards needed | Notes |
|-------|-------------|-------|
| 0 | 3-5 | P3 stragglers |
| 1 | 5-6 | One card per evasion keyword |
| 2 | 5-7 | One per combat trigger |
| 3 | 4-5 | Including a Ninja for Ninjutsu |
| 4 | 5-6 | One per graveyard-cast ability |
| 5 | 4-5 | One per alt-cast |
| 6 | 5-6 | One per cost modifier |
| 7 | 5-7 | One per spell modifier |
| 8 | 5-6 | One per upkeep mechanic |
| 9 | 5-6 | One per counter mechanic |
| 10 | 5-7 | One per ETB/dies pattern |
| 11 | 4-5 | Modal cards |
| 12 | 4-5 | One per ability word |
| 13 | 6-8 | One per new mechanic |
| 14 | 4-5 | Cipher card, etc. |
| 15 | 0 | Deck validation only |
| Mutate | 3 | Gemrazer, Nethroi, Brokkos |
| **Total** | **~73-93** | On top of existing 111 |

**Strategy**: Manual batch card authoring (Option C from coordination doc).
Write 5-8 card definitions per session at start of each batch. ~10 min per simple
card, ~20 min per complex card.

---

## LegalActionProvider Updates

After every 3-4 batches, update `crates/simulator/src/legal_actions.rs` to
recognize new keywords. This keeps TUI gameplay current without per-ability overhead.

Recommended update points:
- After Batch 3 (evasion + combat triggers + Ninjutsu)
- After Batch 6 (alt costs + cost mods)
- After Batch 10 (ETB/dies + counters)
- After Batch 14 (remaining niche)

---

## Progress Tracking

| Batch | Size | Est. Sessions | Status |
|-------|------|--------------|--------|
| 0: P3 Stragglers | 5 | 1 | **Complete** |
| 1: Evasion & Simple | 6 | 1 | **Complete** |
| 2: Combat Triggers (Blocking) | 7 | 1-2 | **Complete** |
| 3: Combat Modifiers & Ninjutsu | 5 | 1-2 | **Complete** |
| 4: Alt-Cast Graveyard | 6 | 1-2 | **Complete** |
| 5: Alt-Cast Hand/Exile | 5 | 1 | **Complete** |
| 6: Cost Modification | 6 | 1-2 | **Complete** |
| 7: Spell Modifiers | 7 | 2 | Not started |
| 8: Upkeep, Time & Phasing | 7 | 2-3 | Not started |
| 9: Counter & Growth | 6 | 1 | Not started |
| 10: ETB/Dies Patterns | 7 | 2 | Not started |
| 11: Modal Choice + Deps | 5 | 2 | Not started |
| 12: Ability Words | 5 | 1 | Not started |
| 13: Newer Set Mechanics | 8 | 2 | Not started |
| 14: Niche & Encoding | 6 | 2 | Not started |
| 15: Commander Variants | 3 | 0.5 | Not started |
| Mutate Mini-Milestone | 1 | 2-3 | Not started |
| **Total** | **~93** | **~23-28** | |

---

## After All Batches + Mutate

When all implementable abilities + Mutate are done:

1. Run `/audit-abilities` to refresh coverage doc
2. Update `docs/workstream-coordination.md` Phase 1 checkboxes
3. Decide whether to tackle remaining blocked subsystems (Morph tree 5, Transform tree 4)
   as a dedicated pre-M10 milestone or defer to post-M10
4. Update LegalActionProvider one final time
5. Run fuzzer with expanded ability pool to find interaction bugs
6. Proceed to Phase 2 (TUI hardening) → Phase 3 (LOWs) → Phase 4 (M10)
