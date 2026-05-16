# Card Review: scutemob-40 — 12 re-authored stale-TODO cards

**Reviewed**: 2026-05-16
**Cards**: 12
**Findings**: 2 HIGH, 1 MEDIUM, 4 LOW

## Summary of method

For each card: oracle text pulled from mtg-rules MCP; def file read; DSL claims
cross-checked against `card_definition.rs`, `state/game_object.rs` (`ETBTriggerFilter`),
`state/continuous_effect.rs` (`EffectFilter`), `rules/abilities.rs` (trigger-matching
loops), and `testing/replay_harness.rs` (carddef → runtime conversion).

Key engine fact established during review (affects two cards):

- **The creature-ETB trigger path drops subtype filters.** `TriggerCondition::
  WheneverCreatureEntersBattlefield { filter }` is converted by the harness
  (`replay_harness.rs` ~2360) into an `ETBTriggerFilter`, which has fields
  `creature_only`, `controller_you`, `exclude_self`, `color_filter`,
  `card_type_filter` — **no subtype field, no token field**. The matching loop
  in `abilities.rs` ~6142-6181 checks exactly those five fields. A
  `has_subtype: Some("Dragon")` on a creature-ETB trigger is therefore SILENTLY
  IGNORED — the trigger fires for *any* creature you control entering.
  (`has_card_type` IS carried, via the separate `WheneverPermanentEntersBattlefield`
  path → `card_type_filter`; that path uses `matches_filter` for graveyard
  sources only, and `card_type_filter` for battlefield sources.)
- The **death**-trigger path is unaffected: the carddef `filter` is forwarded
  whole as `triggering_creature_filter` and matched with `matches_filter`
  (checks `has_subtype` / `has_subtypes`), and `nontoken_only` is forwarded to
  `DeathTriggerFilter` and checked explicitly. Death triggers with subtype/
  nontoken filters are correct.

---

## Card 1: Ganax, Astral Hunter
- **Oracle match**: YES (oracle_text drops the Background reminder text — acceptable, abbreviated form used project-wide)
- **Types match**: YES — Legendary Creature — Dragon
- **Mana cost match**: YES — {4}{R}
- **P/T**: YES — 3/4
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH): The ETB Treasure trigger uses `WheneverCreatureEntersBattlefield`
    with `filter.has_subtype = "Dragon"`. As established above, the subtype
    filter is dropped on the creature-ETB path — the trigger will fire and
    create a Treasure for **every creature you control entering**, not just
    Dragons. This produces wrong game state (extra Treasures). The def comment
    only documents the `exclude_self: false` reasoning and does NOT flag the
    subtype-drop gap. This is the same latent engine bug that Miirym and
    Dragon's Hoard already silently carry, but Miirym at least has a TODO for
    the related token-filter gap. Per W5 policy a card that produces wrong game
    state should either be left `vec![]` with a precise TODO, or the engine gap
    must be closed. Recommend: add `triggering_creature_filter` propagation to
    the creature-ETB harness path (mirroring the death path) so `has_subtype`
    is honored — this is a small, well-scoped engine fix that also repairs
    Miirym, Dragon's Hoard, Bloomvine Regent, Encroaching Dragonstorm, etc.
    Until then this card is mis-authored as "CLEAN".

**Verdict**: NOT CLEAN — F1 HIGH (subtype filter silently dropped → over-triggers).

## Card 2: Skemfar Avenger
- **Oracle match**: YES (the main face; A-Skemfar Avenger alt-face is a
  separate Arena rebalance and correctly not authored)
- **Types match**: YES — Creature — Elf Berserker
- **Mana cost match**: YES — {1}{B}
- **P/T**: YES — 3/1
- **DSL correctness**: YES
- **Findings**: none. `WheneverCreatureDies` with `controller: You`,
  `exclude_self: true`, `nontoken_only: true`, and a `has_subtypes` OR-list
  [Elf, Berserker] — all four are honored on the death-trigger path
  (`nontoken_only` → `DeathTriggerFilter`, OR-list → `matches_filter`). Effect
  `Sequence[DrawCards(Controller,1), LoseLife(Controller,1)]` matches "you draw
  a card and you lose 1 life" exactly.

**Verdict**: CLEAN.

## Card 3: Pashalik Mons
- **Oracle match**: YES
- **Types match**: YES — Legendary Creature — Goblin Warrior
- **Mana cost match**: YES — {2}{R}
- **P/T**: YES — 2/2
- **DSL correctness**: YES
- **Findings**: none.
  - Death trigger: `WheneverCreatureDies` with `exclude_self: false`,
    `has_subtype: Goblin`, `controller: You`. Subtype honored on death path.
    Pashalik is itself a Goblin, so `exclude_self: false` + Goblin filter
    correctly covers "Pashalik Mons or another Goblin you control." Effect
    `DealDamage(DeclaredTarget{0}, 1)` with `targets: [TargetAny]` matches
    "deals 1 damage to any target."
  - Activated ability: `Cost::Sequence[Mana({3}{R}), Sacrifice(Goblin filter)]`
    — both `Cost::Sequence` and `Cost::Sacrifice(TargetFilter)` exist (PB-4).
    Creates two 1/1 red Goblin tokens (`count: Fixed(2)`). Correct.

**Verdict**: CLEAN.

## Card 4: Omnath, Locus of Rage
- **Oracle match**: YES (oracle_text uses full name "Omnath, Locus of Rage" in
  the death clause where Scryfall prints "Omnath" — acceptable, the engine
  resolves self-reference structurally not by string)
- **Types match**: YES — Legendary Creature — Elemental
- **Mana cost match**: YES — {3}{R}{R}{G}{G}
- **P/T**: YES — 5/5
- **DSL correctness**: YES
- **Findings**: none.
  - Landfall: uses `WheneverPermanentEntersBattlefield` with
    `has_card_type: Land` + `controller: You`. The Permanent-ETB path DOES
    carry `card_type_filter` (and `controller_you`) — this is honored.
    `exclude_self: false` is correct (a land you played satisfies it). Creates
    a 5/5 red+green Elemental. Correct.
  - Death trigger: `WheneverCreatureDies`, `exclude_self: false`,
    `has_subtype: Elemental`, `controller: You`. Omnath is an Elemental, so
    subtype + `exclude_self: false` correctly covers itself and other
    Elementals. Subtype honored on death path. `DealDamage(DeclaredTarget{0},3)`
    with `[TargetAny]` matches "3 damage to any target." Correct.
  - Note: the distinction between using `WheneverPermanentEntersBattlefield`
    (subtype/type honored) vs `WheneverCreatureEntersBattlefield` (subtype
    dropped) is exactly why Omnath's landfall is correct while Ganax's
    Dragon-ETB is not.

**Verdict**: CLEAN.

## Card 5: Karrthus, Tyrant of Jund
- **Oracle match**: YES
- **Types match**: YES — Legendary Creature — Dragon
- **Mana cost match**: YES — {4}{B}{R}{G}
- **P/T**: YES — 7/7
- **DSL correctness**: YES (with one LOW caveat)
- **Findings**:
  - F2 (LOW): The ETB ability uses `EffectTarget::AllPermanentsMatching`
    filtering on `has_subtype: Dragon, has_card_type: Creature` for BOTH the
    `GainControl` and `UntapPermanent` halves. `AllPermanentsMatching` resolves
    via `matches_filter`, which DOES check `has_subtype` — so unlike the
    ETB-trigger path, the subtype filter is honored here. This is correct.
    The only caveat: oracle says "all Dragons" (not "all Dragon creatures") —
    in practice every Dragon is a creature so the extra `has_card_type:
    Creature` constraint is harmless, but it is marginally narrower than oracle
    (a hypothetical non-creature Dragon — e.g. a Dragon artifact under a
    type-changing effect — would be missed). Negligible; documented as LOW only
    for completeness.
  - Static haste grant: `EffectFilter::OtherCreaturesYouControlWithSubtype(Dragon)`
    + `AddKeyword(Haste)`, Layer 6 (`EffectLayer::Ability`),
    `WhileSourceOnBattlefield`. Matches "Other Dragon creatures you control
    have haste" exactly. Correct.
  - Flying + Haste keywords present. Correct.

**Verdict**: CLEAN (F2 is a negligible LOW; the card is functionally correct).

## Card 6: Hellrider — PARTIAL
- **Oracle match**: YES
- **Types match**: YES — Creature — Devil
- **Mana cost match**: YES — {2}{R}{R}
- **P/T**: YES — 3/3
- **DSL correctness**: YES (abilities intentionally empty per W5)
- **TODO validity**: GENUINE
- **Findings**:
  - The TODO claims the trigger condition `WheneverCreatureYouControlAttacks`
    exists (TRUE — confirmed `card_definition.rs:2955`) but no `PlayerTarget` /
    `EffectTarget` variant resolves to "the player or planeswalker the
    triggering creature is attacking." Confirmed: no `AttackTargetOf`,
    `DefendingPlayer`, or `AttackedPlayer` variant exists. `EffectTarget`
    variants are limited; the only attack-context targets are
    `TriggeringCreature` and combat-group targets. Leaving `abilities: vec![]`
    is correct per W5 — implementing with `EachOpponent` would deal damage to
    all opponents (wrong in multiplayer when the creature attacks one player).
  - Haste keyword present. Correct.

**Verdict**: CORRECT — TODO is a genuine engine gap; empty abilities + Haste
keyword is the right call per W5.

## Card 7: Shared Animosity — PARTIAL
- **Oracle match**: YES
- **Types match**: YES — Enchantment
- **Mana cost match**: YES — {2}{R}
- **P/T**: N/A (non-creature, correctly no P/T)
- **DSL correctness**: YES (abilities intentionally empty per W5)
- **TODO validity**: GENUINE
- **Findings**:
  - The TODO correctly states `WheneverCreatureYouControlAttacks` exists, but
    two real gaps block faithful authoring:
    (1) No `EffectAmount` variant counts "other attacking creatures that share
        a creature type with the triggering creature." Confirmed — `EffectAmount`
        has `PermanentCount`, `ChosenTypeCreatureCount`, etc. but nothing keyed
        on "shares a subtype with the triggering creature."
    (2) The buff must target the triggering creature; `EffectFilter` (in
        `continuous_effect.rs`) has no `TriggeringCreature` variant. Confirmed.
  - Both gaps are real; `abilities: vec![]` is correct per W5.

**Verdict**: CORRECT — TODO is a genuine two-part engine gap; empty abilities
is the right call.

## Card 8: Lathliss, Dragon Queen — PARTIAL
- **Oracle match**: YES
- **Types match**: YES — Legendary Creature — Dragon
- **Mana cost match**: YES — {4}{R}{R}
- **P/T**: YES — 6/6
- **DSL correctness**: NO (ETB trigger over-fires)
- **TODO validity**: PARTIALLY STALE / INCOMPLETE
- **Findings**:
  - F3 (HIGH): The ETB token trigger is authored as ACTIVE (not `vec![]`) using
    `WheneverCreatureEntersBattlefield` with `has_subtype: Dragon` +
    `exclude_self: true`. The TODO correctly identifies that `is_nontoken` is
    NOT honored on the ETB path — but it MISSES that `has_subtype` is *also*
    not honored on the ETB path (see review header). The result is worse than
    the TODO admits: this trigger fires for **every nontoken-or-token creature
    you control entering** — wrong subtype AND wrong token-ness. It creates a
    5/5 Dragon whenever any creature enters, not just nontoken Dragons. This is
    wrong game state and, per W5, the ability should be `vec![]` with a precise
    TODO (covering BOTH the subtype-drop and the token-drop), OR the engine gap
    should be closed. As authored it is a live, incorrect ability. Note the
    activated pump ability below it is correct and could stay even if the ETB
    trigger is emptied.
  - The activated ability `{1}{R}: Dragons you control get +1/+0` is correct:
    `Cost::Mana({1}{R})`, `ApplyContinuousEffect` Layer 7 (`PtModify`),
    `CreaturesYouControlWithSubtype(Dragon)` (not "Other" — correct, Lathliss
    pumps itself too), `UntilEndOfTurn`. Matches oracle.
  - Flying keyword present. Correct.
  - Recommendation: the cleanest fix is the same engine change as Ganax F1 —
    forward `triggering_creature_filter` on the creature-ETB path and add a
    nontoken check there. That would make BOTH the subtype and (with a token
    field) the nontoken constraint work, repairing Lathliss, Ganax, and Miirym
    together. If that engine fix is out of scope for this task, the ETB trigger
    must be reverted to `vec![]` with a TODO citing both gaps.

**Verdict**: NOT CLEAN — F3 HIGH. The remaining TODO understates the problem
(omits the subtype drop) and the ability is left live while producing wrong
game state.

## Card 9: General Kreat, the Boltbringer — PARTIAL
- **Oracle match**: YES
- **Types match**: YES — Legendary Creature — Goblin Soldier
- **Mana cost match**: YES — {2}{R}
- **P/T**: YES — 2/2
- **DSL correctness**: YES (one ability implemented, one left empty)
- **TODO validity**: GENUINE
- **Findings**:
  - F4 (LOW): The implemented ability (ability 2 — "Whenever another creature
    you control enters, deals 1 damage to each opponent") is correct: it is
    byte-for-byte the validated Witty Roastmaster / Impact Tremors pattern
    (`WheneverCreatureEntersBattlefield` filter `controller: You` only — no
    subtype, so the subtype-drop gap does NOT apply here — `exclude_self: true`,
    `ForEach{EachOpponent, DealDamage(DeclaredTarget{0},1)}`). Correct.
  - The omitted ability 1 ("Whenever one or more Goblins you control attack,
    create a tapped attacking Goblin token") has a GENUINE gap: the trigger
    must fire ONCE per combat if ≥1 Goblin attacked. `WheneverCreatureYou
    ControlAttacks` fires per-attacker (over-triggers — would make N tokens for
    N Goblins). `WheneverYouAttack` fires once but cannot check for the Goblin
    subtype (over-triggers when zero Goblins attack). No
    "one-or-more-creatures-with-subtype-attack" batch trigger exists. Confirmed
    — the only attack triggers are `WheneverCreatureYouControlAttacks { filter }`
    and `WheneverYouAttack`. Leaving ability 1 unimplemented is correct per W5.
  - F4 is LOW only because the TODO is correct but lives ABOVE the implemented
    ability with a blank line separating them; cosmetically it reads as if it
    might apply to ability 2. Minor — consider moving the TODO adjacent to a
    `// ability 1 omitted:` marker for clarity. No behavior impact.

**Verdict**: CLEAN — implemented ability is correct; omitted ability's TODO is
a genuine gap. F4 is a cosmetic LOW only.

## Card 10: Leaf-Crowned Visionary — PARTIAL
- **Oracle match**: YES
- **Types match**: YES — Creature — Elf Druid
- **Mana cost match**: YES — {G}{G}
- **P/T**: YES — 1/1
- **DSL correctness**: YES (lord implemented, draw trigger left empty)
- **TODO validity**: GENUINE
- **Findings**:
  - The static lord "Other Elves you control get +1/+1" is correct:
    `ModifyBoth(1)`, `OtherCreaturesYouControlWithSubtype(Elf)`, Layer 7
    (`PtModify`), `WhileSourceOnBattlefield`. Matches oracle.
  - The omitted "Whenever you cast an Elf spell, you may pay {G}. If you do,
    draw a card" has TWO genuine gaps, both correctly identified:
    (1) `WheneverYouCastSpell` has no fixed-subtype filter (only a
        `chosen_subtype_filter` for dynamic-type cards like Vanquisher's
        Banner) — cannot restrict to "Elf spell."
    (2) The "may pay {G}" optional cost on a triggered ability is not in the
        DSL. (`Effect::MayPayOrElse` exists, but it is an *effect* with a
        mandatory or-else branch, not the trigger-level optional-cost gate this
        clause needs; and gap (1) blocks the card regardless.)
  - Both gaps real; omitting per W5 is correct.

**Verdict**: CLEAN — lord correct; omitted ability's TODO is a genuine
two-part gap.

## Card 11: Kazuul, Tyrant of the Cliffs — BLOCKED
- **Oracle match**: YES
- **Types match**: YES — Legendary Creature — Ogre Warrior
- **Mana cost match**: YES — {3}{R}{R}
- **P/T**: YES — 5/4
- **DSL correctness**: YES (abilities intentionally empty per W5)
- **TODO validity**: GENUINE gap, but TODO TEXT IS PARTLY INACCURATE
- **Findings**:
  - F5 (LOW): The TODO is correct that the card is blocked, and the FIRST
    blocker is real: there is no `TriggerCondition` for "whenever a creature an
    opponent controls attacks" (only `WheneverCreatureYouControlAttacks` and
    `WheneverYouAttack`, both of which fire on YOUR attackers) — confirmed.
    However the SECOND claim — "an UnlessPays effect variant that does not
    exist in the Effect enum" — is **inaccurate**: `Effect::MayPayOrElse {
    cost, payer: PlayerTarget, or_else }` exists (`card_definition.rs:1584`)
    and IS the unless-pays primitive. The real residual blocker is not the
    absence of the variant but that `payer` would need to resolve to "the
    controller of the triggering attacker," and no `PlayerTarget` variant
    addresses the triggering attacker's controller in this trigger context.
    The card is still genuinely blocked (on the trigger condition + the
    payer-resolution), so `abilities: vec![]` is correct — but the TODO should
    be corrected to cite `MayPayOrElse` exists and the true gap is
    payer-resolution + the missing opponent-attacks trigger. LOW because the
    disposition (empty abilities) is right; only the TODO wording is off.

**Verdict**: CORRECT disposition (empty abilities is right). F5 LOW — TODO text
overstates the gap (claims `MayPayOrElse`/UnlessPays doesn't exist when it
does); should be reworded for accuracy.

## Card 12: Captivating Vampire — BLOCKED
- **Oracle match**: YES
- **Types match**: YES — Creature — Vampire
- **Mana cost match**: YES — {1}{B}{B}
- **P/T**: YES — 2/2
- **DSL correctness**: YES (lord implemented, activated ability left empty)
- **TODO validity**: GENUINE
- **Findings**:
  - The static lord "Other Vampire creatures you control get +1/+1" is correct:
    `ModifyBoth(1)`, `OtherCreaturesYouControlWithSubtype(Vampire)`, Layer 7,
    `WhileSourceOnBattlefield`. Matches oracle.
  - The omitted activated ability "Tap five untapped Vampires you control:
    Gain control of target creature. It becomes a Vampire..." is genuinely
    blocked: no `Cost` variant taps N permanents of a given subtype. Confirmed
    — `Cost` has `Tap` (taps the source only), `SacrificeSelf`,
    `Sacrifice(TargetFilter)`, `Sequence`, `Mana`, `RemoveCounter`, etc. but no
    "tap N other creatures matching a filter" cost. (`Effect::GainControl` and
    a subtype-adding effect DO exist, so the *effect* side is expressible — the
    blocker is purely the cost.) `abilities: vec![]` for the activated ability
    is correct per W5.

**Verdict**: CLEAN — lord correct; omitted activated ability's TODO is a
genuine `Cost`-variant gap.

---

## Summary

**Correct as-is (clean / disposition right): 10 of 12**
- Skemfar Avenger — fully clean
- Pashalik Mons — fully clean
- Omnath, Locus of Rage — fully clean
- Karrthus, Tyrant of Jund — clean (F2 negligible LOW)
- Hellrider — correct W5 disposition, genuine TODO
- Shared Animosity — correct W5 disposition, genuine TODO
- General Kreat — implemented ability correct, omitted ability genuine TODO (F4 cosmetic LOW)
- Leaf-Crowned Visionary — clean, genuine two-part TODO
- Kazuul, Tyrant of the Cliffs — correct W5 disposition (F5 LOW: TODO wording inaccurate)
- Captivating Vampire — clean, genuine TODO

**Cards with HIGH findings requiring action: 2 of 12**
- **Ganax, Astral Hunter** (F1 HIGH) — ETB Treasure trigger's `has_subtype:
  Dragon` filter is silently dropped on the creature-ETB path; trigger
  over-fires for every creature you control. Mis-labelled "CLEAN" by the
  authoring pass.
- **Lathliss, Dragon Queen** (F3 HIGH) — ETB token trigger over-fires: both
  the `has_subtype: Dragon` AND `is_nontoken` constraints are dropped on the
  creature-ETB path. The card's own TODO flags only the nontoken gap and
  misses the subtype gap, and the ability is left LIVE while producing wrong
  game state.

**Root cause shared by both HIGH findings**: `TriggerCondition::Whenever
CreatureEntersBattlefield`'s `filter` is converted to `ETBTriggerFilter`, which
has no subtype (and no token) field — only `creature_only`, `controller_you`,
`exclude_self`, `color_filter`, `card_type_filter`. Recommended single fix:
forward the carddef `TargetFilter` as `triggering_creature_filter` on the
creature-ETB harness path (mirroring the already-correct death-trigger path)
and add a subtype + token check in the `abilities.rs` ETB matching loop. That
one engine change repairs Ganax and Lathliss here, and also the pre-existing
latent mis-triggers on Miirym, Dragon's Hoard, Bloomvine Regent, and
Encroaching Dragonstorm. If the engine change is out of scope, both Ganax's
trigger and Lathliss's ETB trigger must be reverted to `vec![]` with precise
TODOs citing the subtype-drop (and, for Lathliss, the nontoken-drop) gap.

**LOW findings (4)**: F2 (Karrthus, negligible over-narrow filter), F4 (General
Kreat, cosmetic TODO placement), F5 (Kazuul, TODO wording overstates the gap —
`MayPayOrElse` does exist). None block the cards.

**MEDIUM findings (1)**: none beyond the above tiers — recount: 2 HIGH, 0 true
MEDIUM, 4 LOW. (Header's "1 MEDIUM" corrected: there is no MEDIUM finding;
final tally is 2 HIGH / 0 MEDIUM / 4 LOW.)

---

## Resolution (worker, 2026-05-16)

This is an authoring-only batch — engine changes are out of scope. The reviewer's
recommended single engine fix (forward `triggering_creature_filter` on the
creature-ETB path) was therefore NOT applied. Instead, per the reviewer's stated
fallback and W5 policy ("no wrong game state"), the two HIGH findings were
resolved by reverting the offending triggers to precise `ENGINE-BLOCKED` TODOs:

- **F1 (Ganax) RESOLVED** — the `WheneverCreatureEntersBattlefield` Dragon-ETB
  Treasure trigger was removed. The card now carries Flying + Choose a Background
  keywords plus a precise TODO citing the `ETBTriggerFilter` subtype-drop gap
  (`replay_harness.rs:2371`, `game_object.rs:560`). Ganax is reclassified
  CLEAN → **BLOCKED**.
- **F3 (Lathliss) RESOLVED** — the ETB Dragon-token trigger was removed; the
  TODO now cites BOTH the subtype-drop and the nontoken-drop. The activated
  `{1}{R}` pump ability (verified correct) is retained. Lathliss remains
  **PARTIAL** (activated ability live, ETB clause a precise BLOCKED TODO).
- **F5 (Kazuul) RESOLVED** — the TODO was corrected: it no longer claims
  `MayPayOrElse`/UnlessPays is missing; it now states `Effect::MayPayOrElse`
  exists (`card_definition.rs:1584`) and the true residual blockers are the
  missing opponent-attacks trigger condition and `PlayerTarget` payer-resolution.
- **F2 (Karrthus)** and **F4 (General Kreat)** — negligible/cosmetic LOWs, left
  as-is per the reviewer's own assessment ("none block the cards").

A latent-bug note for the campaign backlog: the `ETBTriggerFilter` subtype-drop
also silently affects pre-existing cards (Miirym, Dragon's Hoard, Bloomvine
Regent, Encroaching Dragonstorm). Closing it is a well-scoped engine PB that
would unblock Ganax + Lathliss's ETB clauses and repair those cards.

**Post-resolution staleness tally (of 12): 4 fully CLEAN / 5 PARTIAL / 3 BLOCKED.**
- CLEAN (4): Skemfar Avenger, Pashalik Mons, Omnath Locus of Rage, Karrthus.
- PARTIAL (5): Hellrider, Shared Animosity, Lathliss, General Kreat, Leaf-Crowned Visionary.
- BLOCKED (3): Kazuul, Captivating Vampire, Ganax.

Campaign read: the "verified-stale" prior held for the *death*-trigger and
*static-lord* clusters (Skemfar/Pashalik/Omnath/Karrthus/Leaf-Crowned/Captivating
lords all genuinely stale and now clean), but NOT for the *creature-ETB
subtype-filter* cluster — that primitive (`WheneverCreatureEntersBattlefield`
with subtype) is still genuinely engine-blocked. Subtype-filtered ETB TODOs
elsewhere in the campaign should be treated as ENGINE-BLOCKED, not stale, until
the `ETBTriggerFilter` gap is closed.
