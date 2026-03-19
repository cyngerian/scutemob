# Primitive Batch Review: PB-17 -- Library Search Filters

**Date**: 2026-03-19
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: CR 701.23 (Search), CR 202.3 (Mana Value), CR 305.6/305.8 (Basic Land Types)
**Engine files reviewed**: `cards/card_definition.rs` (TargetFilter), `effects/mod.rs` (matches_filter + SearchLibrary dispatch), `state/hash.rs` (TargetFilter HashInto), `cards/helpers.rs` (basic_land_filter)
**Card defs reviewed**: 27 files using SearchLibrary
**Test file reviewed**: `tests/library_search.rs` (6 tests)

## Verdict: needs-fix

PB-17 extended TargetFilter with `max_cmc`, `min_cmc`, `has_card_types`, and `has_name` fields
instead of creating separate SearchFilter enum variants. This was a better design choice -- the
generic filter struct is more composable. The `matches_filter` function correctly handles all
new fields. Hash support is complete. However, there are 4 HIGH issues (wrong game state in
card defs), 4 MEDIUM issues, and 1 LOW issue.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 8 | MEDIUM | multiple files | **Wrong CR citation for Search.** All SearchLibrary comments cite CR 701.19 (Regenerate) instead of CR 701.23 (Search). **Fix:** Find/replace `CR 701.19` references in SearchLibrary contexts to `CR 701.23`. Leave Regenerate references as CR 701.19. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 1 | **HIGH** | `vampiric_tutor.rs` | **Shuffle undoes top-of-library placement.** Oracle + ruling say "shuffle and put that card on top" (single action, card ends on top). Def does SearchLibrary to Library{Top} then Shuffle, which randomizes the card's position. **Fix:** see details. |
| 2 | **HIGH** | `worldly_tutor.rs` | **Same shuffle-order bug as Vampiric Tutor.** Oracle: "shuffle and put the card on top." Def: SearchLibrary to Top then Shuffle. **Fix:** same as Finding 1. |
| 3 | **HIGH** | `assassins_trophy.rs` | **Wrong target filter -- two errors.** Oracle: "Destroy target permanent an opponent controls." Def target: `non_land: true` (nonland only) with no controller filter. Oracle has no nonland restriction and requires opponent control. **Fix:** see details. |
| 4 | **HIGH** | `prismatic_vista.rs` | **Land enters tapped but oracle says untapped.** Oracle: "put it onto the battlefield, then shuffle." No "tapped." Def uses `tapped: true`. **Fix:** change `tapped: true` to `tapped: false`. |
| 5 | MEDIUM | `boseiju_who_endures.rs` | **Search filter too narrow.** Oracle: "land card with a basic land type." This means any land with Plains/Island/Swamp/Mountain/Forest subtype (includes shock lands, duals). Def uses `basic: true` which requires Basic supertype. **Fix:** see details. |
| 6 | MEDIUM | `crop_rotation.rs` | **Missing Shuffle.** Oracle: "put that card onto the battlefield, then shuffle." Def has only SearchLibrary, no Shuffle step after. **Fix:** wrap in Sequence with Shuffle. |
| 7 | MEDIUM | `kodamas_reach.rs` | **Missing Arcane subtype.** Oracle type line: "Sorcery -- Arcane." Def uses `types(&[CardType::Sorcery])` with no Arcane subtype. **Fix:** use `types_sub(&[CardType::Sorcery], &["Arcane"])`. |
| 8a | MEDIUM | `urzas_saga.rs` | **Filter checks mana value, not literal mana cost.** Oracle: "artifact card with mana cost {0} or {1}." Ruling: "you can find only a card with actual mana cost {0} or {1}, not mana value 0 or 1." Def uses `max_cmc: Some(1)` which matches by mana value. A card like Sol Ring ({1}) matches correctly, but a card like Springleaf Drum ({1}) also matches, while Mox Opal ({0}) matches. However, Mishra's Bauble (mana cost {0}) matches and Mana Crypt (mana cost {0}) matches, but Tormod's Crypt also matches -- all correct for MV. The real issue: {U} cards (MV 1) would wrongly match. **Fix:** see details. |
| 9 | LOW | multiple | **Known deferred TODOs.** Tiamat (multi-card search), Goblin Ringleader (reveal-route), Finale of Devastation (graveyard + X pump), Scion (copy-self), Inventors' Fair (activation condition + upkeep trigger), Harald (look-at-top-N), Boseiju (target filter + cost reduction), Urza's Saga (chapters I-II), Crop Rotation (additional cost). All documented with TODO comments. |

### Finding Details

#### Finding 1: Vampiric/Worldly Tutor shuffle order

**Severity**: HIGH
**Files**: `crates/engine/src/cards/defs/vampiric_tutor.rs:13-22`, `crates/engine/src/cards/defs/worldly_tutor.rs:13-27`
**Oracle**: Vampiric: "Search your library for a card, then shuffle and put that card on top."
**Ruling (2016-06-08)**: "The 'shuffle and put the card on top' is a single action."
**Issue**: The def sequences SearchLibrary (dest: Library Top) then Shuffle. The card is placed
on top first, then the entire library (including the card) is shuffled, randomizing the card's
position. The oracle requires the card to end up on top after the shuffle.
**Fix**: The DSL cannot currently express "search, set aside, shuffle, then place on top" as a
single atomic operation. Workaround: change SearchLibrary destination to `ZoneTarget::Hand`
(temp staging), then Shuffle, then add a new PutOnLibrary-from-hand effect. Alternatively, add
a SearchLibrary flag `shuffle_before_placing: bool` that shuffles the library after finding the
card but before moving it to the destination. The flag approach is cleaner and avoids the card
briefly being in hand (which could trigger "whenever a card is put into your hand" effects).

#### Finding 3: Assassin's Trophy target filter

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/assassins_trophy.rs:42-45`
**Oracle**: "Destroy target permanent an opponent controls."
**Issue**: Target filter has `non_land: true` (excludes lands) but oracle targets any permanent.
Also missing `controller: TargetController::Opponent`. Two compounding errors: (1) can't target
opponent's lands, (2) can target own permanents.
**Fix**: Change target to `TargetRequirement::TargetPermanentWithFilter(TargetFilter { controller: TargetController::Opponent, ..Default::default() })`. Remove `non_land: true`.

#### Finding 4: Prismatic Vista enters tapped

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/prismatic_vista.rs:26`
**Oracle**: "Search your library for a basic land card, put it onto the battlefield, then shuffle."
**Issue**: `destination: ZoneTarget::Battlefield { tapped: true }` but oracle has no "tapped"
qualifier. The fetched land should enter untapped. This is distinct from Evolving Wilds/
Terramorphic Expanse which DO say "tapped."
**Fix**: Change `tapped: true` to `tapped: false` at line 26.

#### Finding 5: Boseiju search filter (basic vs basic land type)

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/boseiju_who_endures.rs:53-57`
**Oracle**: "That player may search their library for a land card with a basic land type"
**CR 305.8**: "Any land with the supertype 'basic' is a basic land. Any land that doesn't have
this supertype is a nonbasic land, even if it has a basic land type."
**Issue**: Filter uses `basic: true` which requires the Basic supertype. Oracle says "a land card
with a basic land type" which means any land having Plains, Island, Swamp, Mountain, or Forest
as a subtype. This includes nonbasic lands like shock lands (Steam Vents = Island Mountain).
**Fix**: Replace `basic: true` with `has_subtypes: vec![SubType("Plains".into()), SubType("Island".into()), SubType("Swamp".into()), SubType("Mountain".into()), SubType("Forest".into())]`. Keep `has_card_type: Some(CardType::Land)`.

#### Finding 6: Crop Rotation missing shuffle

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/crop_rotation.rs:15-23`
**Oracle**: "Search your library for a land card, put that card onto the battlefield, then shuffle."
**Issue**: The effect is a bare `Effect::SearchLibrary { ... }` without a Shuffle step. Oracle
requires shuffle after search.
**Fix**: Wrap in `Effect::Sequence(vec![Effect::SearchLibrary { ... }, Effect::Shuffle { player: PlayerTarget::Controller }])`.

#### Finding 7: Kodama's Reach missing Arcane

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/kodamas_reach.rs:9`
**Oracle type**: "Sorcery -- Arcane"
**Issue**: `types(&[CardType::Sorcery])` omits the Arcane subtype. This matters for Splice onto
Arcane interactions (CR 702.47).
**Fix**: Change to `types_sub(&[CardType::Sorcery], &["Arcane"])`.

#### Finding 8a: Urza's Saga mana cost vs mana value

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/urzas_saga.rs:46`
**Oracle**: "Search your library for an artifact card with mana cost {0} or {1}"
**Ruling (2021-06-18)**: "you can find only a card with actual mana cost {0} or {1}, not mana
value 0 or 1. For example, you couldn't find a card with mana cost {U} or one with mana cost {X}."
**Issue**: `max_cmc: Some(1)` filters by mana value, not literal mana cost. Cards with colored
mana costs (e.g., Aether Vial at {1} is correct, but Ancestral Vision at {suspend-only} = MV 0
would also match). More importantly, a hypothetical 1-colored-mana artifact ({U}) would wrongly
pass the filter.
**Fix**: This is a DSL gap -- TargetFilter has no "exact mana cost" field. Add a
`exact_mana_cost: Option<Vec<ManaCost>>` field to TargetFilter to express "mana cost must be
exactly one of these values." For Urza's Saga: `exact_mana_cost: Some(vec![ManaCost::default(), ManaCost { generic: 1, ..Default::default() }])`. Low urgency since most 0-1 MV artifacts in Commander have generic mana costs anyway, but technically wrong per ruling.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 701.23 (Search) | Yes | Yes | 6 tests in library_search.rs; but all cite wrong CR 701.19 |
| 701.23a (look at all cards) | Yes | Implicit | matches_filter scans entire library |
| 701.23b (not required to find) | Partial | No | Deterministic fallback always finds; M10 interactive choice needed |
| 701.23e (reveal) | Yes | No | `reveal` field exists but no test checks reveal behavior |
| 701.23f (search restriction) | Yes | Yes | Aven Mindcensor top-4 in token_damage_search_replacement.rs |
| 202.3 (mana value) | Yes | Yes | max_cmc/min_cmc tests; MV 0 for no-cost cards tested |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| demonic_tutor | Yes | 0 | Yes | Clean |
| vampiric_tutor | No | 0 | **No** | Shuffle undoes top placement (Finding 1) |
| worldly_tutor | No | 0 | **No** | Same shuffle issue (Finding 2) |
| assassins_trophy | No | 0 | **No** | Wrong target filter (Finding 3) |
| prismatic_vista | No | 0 | **No** | Enters tapped, should be untapped (Finding 4) |
| boseiju_who_endures | No | 4 | **No** | Search filter too narrow (Finding 5) + TODOs |
| crop_rotation | No | 2 | **No** | Missing shuffle (Finding 6) + TODO add-cost |
| kodamas_reach | No | 0 | Partial | Missing Arcane subtype (Finding 7) |
| urzas_saga | Partial | 3 | Partial | MV vs mana cost (Finding 8a) + chapter I/II TODOs |
| path_to_exile | Yes | 0 | Yes | Clean |
| ghost_quarter | Yes | 0 | Yes | Clean |
| solemn_simulacrum | Yes | 0 | Yes | Clean |
| sakura_tribe_elder | Yes | 0 | Yes | Clean |
| wayfarers_bauble | Yes | 0 | Yes | Clean |
| evolving_wilds | Yes | 0 | Yes | Clean |
| terramorphic_expanse | Yes | 0 | Yes | Clean |
| rampant_growth | Yes | 0 | Yes | Clean |
| cultivate | Yes | 0 | Yes | Clean |
| explosive_vegetation | Yes | 0 | Yes | Clean |
| aven_mindcensor | Yes | 0 | Yes | Clean |
| maelstrom_of_spirit_dragon | Yes | 0 | Yes | Clean |
| scion_of_the_ur_dragon | Partial | 2 | Partial | Search OK, copy-self TODO (known deferred) |
| finale_of_devastation | Partial | 4 | Partial | Known deferred (graveyard + X pump) |
| inventors_fair | Partial | 4 | Partial | Known deferred (activation condition + upkeep) |
| tiamat | No | 1 | N/A | ETB entirely missing (known deferred) |
| goblin_ringleader | No | 1 | N/A | ETB entirely missing (known deferred) |
| harald_king_of_skemfar | No | 2 | N/A | ETB entirely missing (known deferred) |

## Test Coverage Assessment

The 6 tests in `library_search.rs` cover:
- max_cmc filter (positive)
- min_cmc filter (positive)
- has_card_types OR semantics
- empty filter (any card)
- combined creature + max_cmc
- no-mana-cost = MV 0
- no-match scenario
- top-of-library destination

Missing test coverage:
- has_subtype filter (Dragon, etc.)
- has_name filter (Partner With search)
- basic_land_filter helper
- reveal flag behavior
- negative: wrong type excluded
- search + shuffle interaction (Vampiric Tutor pattern)
- Aven Mindcensor top-N restriction (tested elsewhere)
