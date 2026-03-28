# Event Log Diagnosability

**Date**: 2026-03-27
**Purpose**: Ensure that when something goes wrong in a game — whether detected by the
runtime integrity checker, noticed by a player, or found in post-game analysis — the
event log contains enough information to diagnose the root cause.

**Related**: `docs/mtg-engine-runtime-integrity.md` (Layer 3: Bug Reporter),
`docs/audits/stress-test-scenarios.md` (scenarios that produce "legal-but-wrong" states)

---

## The Problem

The current `GameEvent` enum records *what happened* but often not *why*. When a player
reports "Thalia's tax didn't apply after Humility resolved," the event log shows:

```
SpellCast { player: 2, ... }           // Opponent cast a noncreature spell
```

But it doesn't show:
- What cost was calculated
- Which cost modifiers were considered
- Whether Thalia's modifier was included or excluded
- Why — what layer-resolved state was checked

For the "legal-but-wrong" bug class (engine is internally consistent, applies wrong rule),
the event log must capture the engine's *reasoning*, not just its *conclusions*.

---

## Current State

`GameEvent` has ~60 variants covering game actions and state changes. What's present:

| Category | Events | Diagnosability |
|----------|--------|---------------|
| Turn structure | TurnStarted, StepChanged, PriorityGiven/Passed | Good — full turn flow visible |
| Spells | SpellCast, SpellResolved, SpellFizzled | Good — includes target info |
| Damage | DamageDealt (source, target, amount) | Good — amount after prevention |
| Zone changes | CardDrawn, PermanentEnteredBattlefield, PermanentDestroyed | Good — includes new ObjectId |
| Triggers | TriggeredAbilityQueued, TriggeredAbilityResolved | Partial — source + ability index but no trigger condition detail |
| SBAs | PermanentDestroyed (via SBA), PlayerLost | Partial — no indication which SBA rule fired |
| Layers | *(none)* | Missing — no events for layer resolution |
| Cost calculation | *(none)* | Missing — no events for how costs were computed |
| Replacement effects | DamagePreventedBy, DamageDoubled | Partial — damage replacements logged, others silent |

---

## Proposed Additions

These are diagnostic events — they don't affect game state but provide an audit trail
for debugging. They should be tagged as diagnostic so the UI can filter them out during
normal play but include them in bug reports.

### Tier 1: Critical for diagnosing known bug classes

**CostCalculated** — emitted during `handle_cast_spell`:
```rust
GameEvent::CostCalculated {
    spell: ObjectId,
    base_cost: ManaCost,
    commander_tax: u32,
    modifiers_applied: Vec<CostModifierApplied>,  // source, change, reason
    final_cost: ManaCost,
}

struct CostModifierApplied {
    source: ObjectId,        // Which permanent provided the modifier
    source_name: String,     // Card name for readability
    change: i32,             // +1 or -1
    filter: String,          // "NonCreature", "CreatureSpell", etc.
    layer_resolved: bool,    // Was this checked through the layer system?
}
```

**SBAFired** — emitted per SBA action in `apply_sbas_once`:
```rust
GameEvent::SBAFired {
    rule: String,            // "CR 704.5f" or "CR 714.4"
    object: ObjectId,
    description: String,     // "Creature toughness <= 0" or "Saga final chapter reached"
}
```

**TriggerEvaluated** — emitted during `collect_triggers_for_event`:
```rust
GameEvent::TriggerEvaluated {
    source: ObjectId,
    source_name: String,
    trigger_condition: String,  // Human-readable trigger description
    event: String,              // What event was being checked
    matched: bool,              // Did the trigger fire?
    suppressed_by: Option<String>,  // "Humility removed ability" / "Torpor Orb"
}
```

### Tier 2: Valuable for complex interactions

**LayerResolution** — emitted when layer resolution produces a notable change:
```rust
GameEvent::LayerResolution {
    object: ObjectId,
    object_name: String,
    layer: String,              // "Layer 6: Ability"
    modification: String,       // "Removed all abilities (Humility)"
    source: ObjectId,           // What continuous effect caused it
}
```

**ReplacementApplied** — emitted for all replacement effects (not just damage):
```rust
GameEvent::ReplacementApplied {
    original_event: String,     // "PermanentEntersBattlefield"
    replacement: String,        // "Enters tapped instead (Thalia Heretic Cathar)"
    source: ObjectId,
    affected: ObjectId,
}
```

### Tier 3: Nice to have for deep debugging

**LKIAccessed** — emitted when the engine falls back to last-known information:
```rust
GameEvent::LKIAccessed {
    object: ObjectId,
    field: String,              // "power", "controller", "keywords"
    value: String,              // The LKI value used
    reason: String,             // "Object no longer on battlefield"
}
```

**KeywordResolution** — emitted when a keyword is checked during combat/triggers:
```rust
GameEvent::KeywordResolution {
    object: ObjectId,
    keyword: String,            // "Flanking", "Exploit"
    present: bool,              // After layer resolution
    source: String,             // "layer-resolved" or "card_registry (BUG)"
}
```

---

## Implementation Strategy

### Phase 1 (with M10 layer bypass fixes)
- Add `SBAFired` — straightforward, one emit per SBA action
- Add `CostCalculated` — emitted in `handle_cast_spell` after cost computation
- Add `TriggerEvaluated` with `suppressed_by` — emitted in `collect_triggers_for_event`

These directly support diagnosing the 9 layer bypass sites. When fixing each site,
add the diagnostic event at the same time.

### Phase 2 (with M10 networking)
- Add `ReplacementApplied` for all replacement types
- Add `LayerResolution` for notable layer changes (sampled, not every object every time)
- Tag all diagnostic events with a `diagnostic: true` flag
- Server filters diagnostics out of normal client broadcasts but includes them in bug reports

### Phase 3 (with M11 UI)
- Bug report captures full event log including diagnostics
- Replay viewer can toggle diagnostic event visibility
- "Why did this happen?" UI overlay shows diagnostic events for a selected game action

### Performance Budget
Diagnostic events are string-heavy. To stay within the 1ms-per-command budget:
- Tier 1 events: always emitted (cheap — one per spell cast, one per SBA)
- Tier 2 events: emitted only in debug mode or when a specific diagnostic flag is set
- Tier 3 events: never in production; test/replay only

---

## How This Helps

**Scenario**: Player reports "Thalia's tax didn't go away after Humility resolved."

**Without diagnostic events**: Event log shows `SpellCast` with no cost breakdown.
Developer must reproduce locally, step through code, guess at what happened.

**With diagnostic events**: Bug report contains:
```
LayerResolution { object: Thalia, layer: "Layer 6", modification: "Removed all abilities", source: Humility }
CostCalculated { spell: Lightning Bolt, modifiers_applied: [
    { source: Thalia, change: +1, filter: "NonCreature", layer_resolved: false }  // ← BUG FLAG
], final_cost: {1}{R} }
```

The `layer_resolved: false` flag immediately identifies the root cause — the cost
modifier was read from the static CardDefinition, not from layer-resolved state.

**Scenario**: Player reports "My Saga died when Blood Moon entered."

**With diagnostic events**:
```
SBAFired { rule: "CR 714.4", object: Urza's Saga, description: "Saga final chapter reached (3 chapter abilities found in CardDefinition)" }
```

The description shows the engine found chapter abilities in the CardDefinition (static),
not in layer-resolved state. Root cause identified.
