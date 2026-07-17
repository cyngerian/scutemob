# Card Review: W-PB2 Batch 1 (target-filter blocker fixes)

**Reviewed**: 2026-07-17
**Cards**: 12
**Findings**: 0 HIGH, 0 MEDIUM, 3 LOW
**Verdict roll-up**: 11 PASS, 1 correctly-marked-`known_wrong` (Patriar's Seal). No FIX required.

Method: oracle text from MCP `lookup_card`; DSL field semantics confirmed against
`crates/card-types/src/cards/card_definition.rs` (`TargetFilter`, lines 2830-2944).
Key confirmations that make the decoy analysis valid:
- `has_card_types: Vec<CardType>` = OR semantics ("at least one"); when combined with the
  singular `has_card_type` BOTH must hold (card_definition.rs:2875-2879).
- `nonbasic: bool` = must lack the Basic supertype (CR 205.4a, line 2850-2854).
- `is_attacking` is a runtime-combat field checked at the target-validation sites for
  `TargetCreatureWithFilter`/`TargetPermanentWithFilter` (line 2913-2931) — not silently dropped.
- `legendary`, `max_cmc`, `has_subtypes` are Characteristics-derived and honored by `matches_filter`.
- `mana_pool(...)` arg order is W,U,B,R,G,C (confirmed across the batch: green=5th, blue=2nd, black=3rd, colorless=6th).

---

## Card 1: Boseiju, Who Endures — PASS
- Oracle match: YES | Types match: YES (Legendary Land) | Mana cost: YES (none) | DSL: YES
- Target filter: "artifact, enchantment, or nonbasic land an opponent controls" →
  `has_card_types:[Artifact,Enchantment,Land], nonbasic:true, controller:Opponent` (lines 77-81).
  Encoding is correct: `nonbasic:true` is a no-op for artifacts/enchantments (they can't carry the
  Basic supertype) and only bites on lands, exactly reproducing "…or *nonbasic* land."
  Decoy check: a basic Forest an opponent controls is rejected on `nonbasic` alone; a nonbasic land
  *you* control is rejected on `controller` alone. Each restriction is independently load-bearing.
- Search sub-effect: `has_card_type:Land + has_subtypes:[Plains,Island,Swamp,Mountain,Forest]`
  correctly models "land card with a basic land type" (CR 305.8) — Bayou/shock lands qualify, plain
  nonbasics do not. Destination untapped (oracle: "put it onto the battlefield"), search player is
  `ControllerOf(target)` = the opponent whose permanent was destroyed. Correct.
- Channel cost reduction indexed at activated-ability 0 (mana tap ability lowered to
  `mana_abilities`), filter `legendary + Creature`, controller = you. Correct.
- Findings:
  - F1 (LOW): Oracle says "That player *may* search … then shuffle"; the def issues an
    unconditional `Effect::Shuffle` after `SearchLibrary`. Benign (a library search shuffles anyway
    per CR 701.19, and finding nothing is always allowed), so game state is not wrong — noted only
    for completeness.

## Card 2: Wasteland — PASS
- Oracle match: YES | Types: YES (Land, non-legendary) | Mana: YES | DSL: YES
- "{T}, Sacrifice this land: Destroy target nonbasic land" → cost `Sequence[Tap, SacrificeSelf]`,
  filter `has_card_type:Land, nonbasic:true` (lines 34-37). Decoy: basic land rejected on
  `nonbasic`; nonland rejected on `has_card_type`. Both independently load-bearing.
- {C} = `mana_pool(0,0,0,0,0,1)` correct.

## Card 3: Minamo, School at Water's Edge — PASS
- Oracle match: YES | Types: YES (Legendary Land) | Mana: YES | DSL: YES
- "{U},{T}: Untap target legendary permanent" → `TargetPermanentWithFilter{legendary:true}`
  (line 38-40). Correctly uses *Permanent* (not Creature). Decoy: nonlegendary permanent rejected on
  `legendary` alone. {U}=`mana_pool(0,1,0,0,0,0)` correct.

## Card 4: Kor Haven — PASS
- Oracle match: YES | Types: YES (Legendary Land) | Mana: YES | DSL: YES
- "Prevent all combat damage that would be dealt *by* target attacking creature" →
  `PreventCombatDamageFromOrTo{prevent_from:true, prevent_to:false}`, filter
  `TargetCreatureWithFilter{is_attacking:true}` (lines 36-45). Direction (from-only) matches oracle.
  Decoy: a non-attacking creature is rejected on `is_attacking` at the validate site. {C} correct.

## Card 5: Maze of Ith — PASS
- Oracle match: YES | Types: YES (Land, non-legendary — correct, Maze is NOT legendary) | DSL: YES
- "{T}: Untap target attacking creature. Prevent all combat damage … to and by that creature" →
  `Sequence[UntapPermanent, PreventCombatDamageFromOrTo{from:true,to:true}]`, filter
  `is_attacking:true` (lines 19-33). Both prevention directions set. Correct.

## Card 6: Shizo, Death's Storehouse — PASS
- Oracle match: YES | Types: YES (Legendary Land) | Mana: YES | DSL: YES
- "{B},{T}: Target legendary creature gains fear until end of turn" → `ApplyContinuousEffect`
  AddKeyword(Fear), `EffectDuration::UntilEndOfTurn`, filter `TargetCreatureWithFilter{legendary:true}`
  (lines 37-50). Decoy: nonlegendary creature rejected on `legendary`. {B} correct.
- Findings:
  - F2 (LOW): Correctness of the granted Fear depends on the engine enforcing Fear evasion in
    combat (CR 702.36) — orthogonal to this def and not verifiable from card-def source. Flagged only
    so the campaign confirms Fear is a `Handled` keyword before trusting the combat outcome.

## Card 7: Patriar's Seal — CORRECTLY known_wrong (no change)
- Oracle match: YES | Types: YES (Artifact) | Mana: YES ({3}) | DSL: partial-by-design
- Newly-added untap ability is CORRECT: "{1},{T}: Untap target legendary creature you control" →
  `TargetCreatureWithFilter{legendary:true, controller:You}` (lines 43-47). Decoy: a legendary
  creature an *opponent* controls is rejected on `controller`; a nonlegendary you control rejected on
  `legendary`.
- The `known_wrong` marker is ACCURATE: the mana ability is `Effect::AddManaAnyColor` (line 21),
  which is a gated stub — it always adds `ManaColor::Colorless` rather than a chosen color, so
  "Add one mana of any color" produces wrong game state. This is exactly the `AddManaAnyColor`
  family gated out of Complete by `effect_choose_gate` (SR-37/SF-11). Keeping it `known_wrong` is the
  right call; it must NOT be upgraded until the per-colour-ability rewire (tainted_field pattern)
  lands for artifacts. Note text is precise and cites the real blocker. VERDICT: correctly marked.

## Card 8: Unearth — PASS
- Oracle match: YES | Types: YES (Sorcery) | Mana: YES ({B}) | DSL: YES
- "Return target creature card with mana value 3 or less from your graveyard to the battlefield" →
  `TargetCardInYourGraveyard{has_card_type:Creature, max_cmc:Some(3)}`, `MoveZone` to battlefield
  untapped (lines 20-32). Decoy: a creature card of MV 4 rejected on `max_cmc`; a noncreature card
  rejected on `has_card_type`. Both independently load-bearing.
- Cycling {2}: carries BOTH `Keyword(Cycling)` marker AND `Cycling{cost}` (lines 33-38) — the
  required dual def. Correct.

## Card 9: Golgari Findbroker — PASS
- Oracle match: YES | Types: YES (Creature — Elf Shaman) | P/T: YES (3/4) | Mana: YES ({B}{B}{G}{G}) | DSL: YES
- "return target permanent card from your graveyard to your hand" →
  `TargetCardInYourGraveyard{has_card_types:[Creature,Artifact,Battle,Enchantment,Land,Planeswalker]}`
  (lines 36-46). The six permanent card types (CR 110.4) with OR semantics == "permanent card";
  instants/sorceries correctly excluded. Decoy: an instant card in graveyard is rejected (absent from
  the list). ETB `WhenEntersBattlefield` → `MoveZone` to Hand(Controller). Correct.

## Card 10: Otawara, Soaring City — PASS
- Oracle match: YES | Types: YES (Legendary Land) | Mana: YES (none) | DSL: YES
- "Return target artifact, creature, enchantment, or planeswalker to its owner's hand" →
  `has_card_types:[Artifact,Creature,Enchantment,Planeswalker]` (lines 50-56). Correctly EXCLUDES
  Battle and Land (contrast Boseiju/Findbroker). Bounce destination is
  `Hand{owner: OwnerOf(target)}` — multiplayer-correct "its *owner's* hand," not Controller.
- Channel cost reduction indexed at activated-ability 0, `legendary + Creature`. Correct.

## Card 11: Mind Games — PASS
- Oracle match: YES | Types: YES (Instant) | Mana: YES ({U}) | DSL: YES
- "Tap target artifact, creature, or land" → `has_card_types:[Artifact,Creature,Land]`,
  `TapPermanent` (lines 30-36). Correct.
- Buyback {2}{U}: `AbilityDefinition::Buyback{cost}` only (no `Keyword` marker) — consistent with the
  existing `constant_mists.rs` convention; Buyback is a cast-time cost modifier carried entirely by
  the `Buyback` variant, unlike Cycling. Not a KI-6 dual-def gap. Cost {2}{U} correct.

## Card 12: Perilous Forays — PASS
- Oracle match: YES | Types: YES (Enchantment) | Mana: YES ({3}{G}{G}) | DSL: YES
- "{1}, Sacrifice a creature: Search … land card with a basic land type … onto the battlefield
  tapped, then shuffle" → cost `Sequence[Mana{generic:1}, Sacrifice(TargetFilter{Creature})]`,
  search `has_card_type:Land + has_subtypes:[5 basic types]`, destination `Battlefield{tapped:true}`,
  then `Shuffle` (lines 22-54). `Cost::Sacrifice(TargetFilter)` is the PB-4 primitive, in scope.
  Destination correctly tapped. Correct.
- Findings:
  - F3 (LOW): Same "basic land type" subtype-list modeling as Boseiju — depends on `SearchLibrary`
    honoring `has_subtypes`; consistent with the Boseiju encoding and no decoy concern.

---

## Summary
- Cards with issues requiring FIX: none.
- Correctly-marked non-Complete: Patriar's Seal (known_wrong — `AddManaAnyColor` gated stub;
  untap half is correct).
- Clean PASS (11): Boseiju Who Endures, Wasteland, Minamo, Kor Haven, Maze of Ith, Shizo,
  Unearth, Golgari Findbroker, Otawara, Mind Games, Perilous Forays.
- LOW notes (non-blocking): F1 unconditional shuffle on Boseiju's optional opponent search;
  F2 confirm Fear is engine-enforced for Shizo; F3 confirm `SearchLibrary` honors `has_subtypes`.
- No gated stub (`Choose`/`MayPayOrElse`/`AddManaChoice`/`AddManaAnyColor`) appears in any
  Complete def in this batch. The only `AddManaAnyColor` use is in Patriar's Seal, which is
  correctly held at `known_wrong`.
