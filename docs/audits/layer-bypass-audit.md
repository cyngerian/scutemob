# Layer Bypass Audit

**Date**: 2026-03-27
**Bug class**: Engine code reads from static `CardDefinition` (via `card_registry`) for
battlefield objects instead of using `calculate_characteristics()` (layer-resolved state).
**Impact**: Any ability-removing effect (Humility, Dress Down, Blood Moon, Overwhelming
Splendor) fails to suppress the bypassed behavior, producing incorrect game results.
**Scheduled fix**: M10 engine correctness pass (pre-networking)

---

## Background

The engine's architecture invariant is that all observable characteristics of battlefield
permanents flow through the layer system (CR 613). `calculate_characteristics()` applies
continuous effects in layer order 1-7, including ability removal at Layer 6.

Nine callsites violate this by reading abilities or ability-derived fields directly from
the `CardDefinition` in the card registry. Under normal conditions this is equivalent to
layer-resolved state. Under ability-removing effects, the static definition still shows
abilities that the layer system has removed, producing incorrect results.

### How to reproduce

Any of these bugs can be triggered by:
1. Having the affected permanent on the battlefield
2. Resolving an ability-removing effect (Humility, Dress Down, etc.)
3. Exercising the affected code path (casting a spell, dealing combat damage, etc.)

The engine will behave as though the ability was never removed.

---

## Findings

### 1. Spell Cost Modifiers (HIGH)

- **File**: `crates/engine/src/rules/casting.rs:5761`
- **Function**: `apply_spell_cost_modifiers()`
- **Reads**: `card_def.spell_cost_modifiers` for all battlefield permanents
- **Bug**: Thalia's "noncreature spells cost {1} more" still applies under Humility.
  Affects all cards with `spell_cost_modifiers`: Thalia Guardian of Thraben, Vryn Wingmare,
  Sphere of Resistance, Goblin Electromancer, Baral Chief of Compliance, Goblin Warchief, etc.
- **Fix**: Resolve cost modifiers from layer-resolved abilities. Requires modeling cost
  modification as a `Static` ability processed through the layer system, or at minimum
  checking `calculate_characteristics()` for the source to confirm it still has abilities.

### 2. Exploit Keyword Count (HIGH)

- **File**: `crates/engine/src/rules/abilities.rs:2436`
- **Function**: ETB trigger generation
- **Reads**: `def.abilities.iter().filter(KeywordAbility::Exploit)` — counts Exploit
  instances from static CardDefinition
- **Bug**: Exploit triggers fire even when the keyword has been removed by Humility/Dress Down.
- **Fix**: Count from `calculate_characteristics(state, obj_id).keywords` instead.

### 3. Flanking Keyword Count (HIGH)

- **File**: `crates/engine/src/rules/abilities.rs:3656`
- **Function**: Combat trigger generation
- **Reads**: `def.abilities.iter().filter(KeywordAbility::Flanking)` — counts Flanking
  instances for stacking -1/-1 triggers
- **Bug**: Flanking triggers fire with wrong count under ability removal.
- **Fix**: Count from layer-resolved keywords.

### 4. Ingest Keyword Count (HIGH)

- **File**: `crates/engine/src/rules/abilities.rs:4332`
- **Function**: Combat damage trigger generation
- **Reads**: `def.abilities.iter().filter(KeywordAbility::Ingest)`
- **Bug**: Ingest triggers fire under ability removal.
- **Fix**: Count from layer-resolved keywords.

### 5. Renown Value Extraction (HIGH)

- **File**: `crates/engine/src/rules/abilities.rs:4386`
- **Function**: Combat damage trigger generation
- **Reads**: `def.abilities.iter().filter_map(KeywordAbility::Renown(n))` — extracts N value
- **Bug**: Renown triggers with wrong N value or fires when removed. Code has a fallback
  to `obj.characteristics.keywords` but uses CardDef as primary source.
- **Fix**: Use layer-resolved keywords as primary (not fallback).

### 6. Poisonous Value Extraction (HIGH)

- **File**: `crates/engine/src/rules/abilities.rs:4443`
- **Function**: Combat damage trigger generation
- **Reads**: `def.abilities.iter().filter_map(KeywordAbility::Poisonous(n))`
- **Bug**: Same as Renown — extracts N from CardDef, has unreliable fallback.
- **Fix**: Use layer-resolved keywords as primary.

### 7. Backup Ability Snapshot (HIGH)

- **File**: `crates/engine/src/rules/abilities.rs:2573`
- **Function**: Backup trigger resolution
- **Reads**: `def.abilities` — snapshots all abilities printed below the Backup keyword
  to grant to the target creature
- **Bug**: If Backup is removed before trigger resolves, code still reads and grants
  printed abilities from CardDef.
- **Fix**: Read from layer-resolved abilities at resolution time. If Backup keyword is
  gone, the trigger should resolve with no effect (CR 603.4 intervening-if pattern).

### 8. Champion Filter Lookup (HIGH)

- **File**: `crates/engine/src/rules/abilities.rs:2629`
- **Function**: Champion ETB trigger
- **Reads**: `def.abilities.iter().find_map(AbilityDefinition::Champion { filter })`
- **Bug**: Champion trigger uses wrong filter or fires when ability removed.
- **Fix**: Look up from layer-resolved abilities.

### 9. Saga SBA Chapter Count (HIGH)

- **File**: `crates/engine/src/rules/sba.rs:765`
- **Function**: `check_saga_sbas()`
- **Reads**: `def.abilities.iter().filter_map(AbilityDefinition::SagaChapter { chapter })`
  — counts chapter abilities from static CardDefinition to determine final chapter number
- **Bug**: Under Blood Moon, Urza's Saga loses all abilities (including chapters) via the
  layer system, but the SBA still finds chapters in the static definition and sacrifices it.
  Per CR 714.4 (updated 2025, Final Fantasy): "a Saga permanent **with one or more chapter
  abilities**" — a Saga with no layer-resolved chapter abilities should not be sacrificed.
- **Fix**: Count chapter abilities from `calculate_characteristics()` resolved abilities,
  not from CardDefinition. If zero chapters found, skip the SBA.

---

## Uniform Fix Strategy

All 9 sites share the same root cause: reading `card_registry.get(card_id).abilities`
(or derived fields) instead of `calculate_characteristics(state, object_id)` for
battlefield permanents.

The fix for each site:
1. Call `calculate_characteristics(state, obj_id)` to get layer-resolved characteristics
2. Read keywords/abilities from the resolved `Characteristics` struct
3. Use that data for the behavior check (trigger generation, cost modification, SBA, etc.)

**Performance note**: `calculate_characteristics()` is already called extensively in SBA
checks, combat, and trigger detection. Adding calls at these 9 sites is negligible.

**Special case — site 1 (cost modifiers)**: `spell_cost_modifiers` is a top-level
`CardDefinition` field, not an ability in `Characteristics`. This requires either:
- (a) Modeling cost modification as a `Static` ability variant in the layer system, or
- (b) Checking that the source permanent still has abilities (via `calculate_characteristics`)
  before applying its cost modifiers. If abilities are empty (Humility), skip.

Option (b) is simpler but less precise — it's an all-or-nothing check. Option (a) is
architecturally correct but touches more code. Recommend (a) for M10.

---

## Cards That Expose These Bugs

Any combination of an ability-removing card + a card at one of the 9 sites:

**Ability removers** (common in Commander):
- Humility — removes all creature abilities
- Dress Down — removes all creature abilities until end of turn
- Blood Moon — removes all non-basic land abilities (replaces with Mountain)
- Overwhelming Splendor — removes all abilities from enchanted player's creatures
- Cursed Totem — removes activated abilities of creatures
- Shadowspear — removes hexproof/indestructible (targeted, not full removal)

**Affected cards** (examples at each site):
1. Cost modifiers: Thalia, Vryn Wingmare, Goblin Electromancer, Baral, Goblin Warchief
2. Exploit: Sidisi Undead Vizier, Silumgar Sorcerer
3. Flanking: Benalish Cavalry, Cavalry Master (stacking flanking)
4. Ingest: Mist Intruder, Fathom Feeder
5. Renown: Consul's Lieutenant, Kytheon Hero of Akros
6. Poisonous: Virulent Sliver (stacking poisonous)
7. Backup: Backup Agent, Boon-Bringer Valkyrie
8. Champion: Changeling Hero, Mistbind Clique
9. Saga SBA: Urza's Saga, Binding the Old Gods (any Saga under Blood Moon)

---

## Status

- [ ] Site 1: Cost modifiers through layer system
- [ ] Site 2: Exploit keyword count
- [ ] Site 3: Flanking keyword count
- [ ] Site 4: Ingest keyword count
- [ ] Site 5: Renown value extraction
- [ ] Site 6: Poisonous value extraction
- [ ] Site 7: Backup ability snapshot
- [ ] Site 8: Champion filter lookup
- [ ] Site 9: Saga SBA chapter count
- [ ] Integration tests: Humility + each affected keyword
- [ ] Integration tests: Blood Moon + Saga
- [ ] Integration tests: Dress Down + cost modifier
