# Scutemob Architecture Review

**Date:** March 26, 2026
**Scope:** Critical analysis of the rules engine architecture as described in `engine_explanation.md`
**Format:** Strengths → Structural Concerns → Fix Strategy → Priority Roadmap

---

## Executive Summary

The engine's foundational architecture is sound. The separation of card definitions as pure data from a generic rules engine is the correct fundamental approach for MTG's combinatorial space, and the implementation demonstrates genuine understanding of both Rust idioms and the Comprehensive Rules. The immutable state design, command/event separation, and layer system are all strong choices that will pay compounding dividends.

This review identifies seven structural concerns. None are insurmountable. Most are additive fixes that elaborate the existing architecture rather than requiring rearchitecture — which is itself the sign of a good architecture. One is a concrete rules bug that should be fixed immediately. The rest are scaling and expressiveness questions that should be addressed in priority order before the alpha release.

---

## Part I: Strengths

### Core Design

The card-data/engine separation is the only approach that has any hope of scaling to MTG's card pool. The document demonstrates this with real conviction, and the concrete examples (Lightning Bolt through Swords to Plowshares) show the DSL composing correctly across a meaningful range of card complexity.

### Immutable State with Structural Sharing

Using `im-rs` persistent data structures for `GameState` is a strong choice. O(1) cloning via structural sharing is essential not just for undo and replay, but eventually for AI search trees if a game-playing agent is ever desired. The `process_command → (GameState, Vec<GameEvent>)` signature makes the engine genuinely testable as a pure function with no hidden state.

### Command/Event Discipline

The invariant that all player actions enter through `Command` and all state changes exit as `GameEvent` is architecturally disciplined in a way that many game engine projects never achieve. This makes the engine a pure library with no I/O coupling — the external layer (network, UI, replay) is cleanly separated.

### Layer System

The `calculate_characteristics` implementation appears faithful to CR 613, including dependency sort and timestamp ordering within layers. Computing characteristics dynamically (rather than caching and invalidating) trades performance for correctness, which is the right tradeoff at this stage. The layer system is one of the hardest parts of an MTG engine, and the approach here is sound.

### SBA Fixed-Point Loop

The fixed-point loop with interleaved trigger checking between SBA passes is correct per CR 704.3. Many implementations get this wrong by either not looping or not checking triggers between passes. The implementation correctly ensures that "when this creature dies" triggers fire even though the creature is removed during the SBA pass.

### Test Coverage

2,300 tests across 1,452 card definitions is a strong foundation. The test-to-card ratio suggests meaningful coverage of interactions, not just isolated card behavior.

---

## Part II: Structural Concerns

### Concern 1: Cost Modifiers Bypass the Layer System (Rules Bug)

**Severity: High — Produces incorrect game results**

Thalia's `spell_cost_modifiers` is modeled as a top-level field on `CardDefinition` rather than as a `Static` ability processed through the layer system. The document explicitly states this is intentional: "Thalia's `spell_cost_modifier` lives on the `CardDefinition` (not as an ability), so it would still apply even under Humility."

This is incorrect per the Comprehensive Rules. Thalia's cost-increasing effect is a static ability (CR 604). Under Humility, Thalia loses all abilities at Layer 6, including the static ability that makes noncreature spells cost more. Any behavioral field on `CardDefinition` that carries game-mechanical semantics but isn't processed through the layer system is a potential source of incorrect results.

This applies to every card with a similar cost-modification effect: Thalia, Vryn Wingmare, Sphere of Resistance, Trinisphere, Goblin Electromancer, Baral, and dozens of others. Every one of them should lose their cost modification under ability-removing effects, and the current architecture prevents that.

### Concern 2: No Interactive Choices During Resolution

**Severity: High — Blocks a large class of cards**

Many spells and abilities require choices *during resolution*, not just during casting:

- "Target opponent chooses a creature they control and sacrifices it"
- "Choose a creature type"
- "Choose one or more"
- "Each player chooses a creature they control, then sacrifices the rest"

The document's resolution model appears to be: pop the stack, execute the effect tree. But the effect tree can contain choice points that depend on game state at resolution time and require player input. These cannot be determined upfront in the `Command` at cast time.

Without a mechanism for resolution to suspend and request player input, a significant number of Commander-playable cards cannot be implemented. This includes staples like Fact or Fiction, Casualties of War, Windfall (where discard counts depend on resolution-time hand sizes), Council's Judgment, and many more.

### Concern 3: Replacement Effect Ordering

**Severity: Medium — Required for rules correctness**

CR 614.5 specifies that when multiple replacement effects could apply to the same event, the affected player or controller chooses which to apply first. The remaining effects may or may not still apply after the first one modifies the event. This creates an interactive choice tree during event processing.

The document shows `replacement_effects: Vector<ReplacementEffect>` as a field on `GameState` but does not describe the protocol for interactive ordering. This is a subset of the choice-during-resolution problem but applies to a different part of the pipeline (event modification rather than effect execution).

### Concern 4: Trigger Condition Taxonomy Scalability

**Severity: Medium — Accumulating maintenance burden**

`TriggerCondition` appears to be an enum with specific variants like `WheneverYouCastSpell` and `WheneverCreatureEntersBattlefield`. This means every mechanically distinct trigger pattern requires its own variant. Patterns that stress a closed taxonomy:

- "Whenever you gain life for the first time each turn" — requires per-turn state tracking per trigger
- "Whenever a player casts a spell, if no mana was spent to cast it" — requires tracking cost payment method
- "At the beginning of each end step, if you gained 3 or more life this turn" — requires per-turn accumulation
- "Whenever you cast a spell from anywhere other than your hand" — requires tracking source zone of cast spells

Each of these needs the trigger system to access specific contextual information that may or may not be present in the `GameEvent`. As more cards are added, the enum grows and the events need to carry more context, creating coupling between the event system and the trigger system.

### Concern 5: Last Known Information (LKI) Mechanism Unspecified

**Severity: Medium — Load-bearing for many interactions**

The Swords to Plowshares example mentions that `EffectTarget::DeclaredTarget` "tracks the object across zone changes via target remapping," but the underlying mechanism isn't described. LKI is one of the most subtle parts of MTG rules (CR 608.2g, 608.2h). When a creature is exiled and an effect references its power, the engine must know the power as it last existed on the battlefield — after all layer system effects were applied. Without a clear LKI model, effects that reference objects in other zones will produce incorrect results.

### Concern 6: DSL Expressiveness Ceiling

**Severity: Low now, High later — Long-term architectural constraint**

The ~60 effect variants handle a wide class of cards, but MTG's design space is adversarial to any fixed DSL. Cards that stress the model:

- **Panglacial Wurm** — casting during a library search, breaking normal Command flow
- **Chains of Mephistopheles** — deeply procedural replacement effect with conditional branching
- **Mindslaver** — turn control requiring Command routing to a different player
- **Necropotence** — step-skipping mechanic not visible in the turn structure
- **Ice Cauldron** — stores mana and spell information across turns

For Commander targeting ~2,000 cards, the DSL is likely sufficient. Beyond ~3,000–4,000 cards with broad coverage, an escape hatch will likely be needed.

### Concern 7: No Discussion of Turn Control or Step Skipping

**Severity: Low — Feature gap, not architectural flaw**

Mindslaver-style turn control and Necropotence-style step skipping are both common in Commander but aren't addressed in the architecture document. These are well-scoped features rather than architectural problems, but they should be on the roadmap.

---

## Part III: Fix Strategy

### Fix 1: Move Cost Modifiers Into the Ability System

**What:** Relocate `spell_cost_modifiers` from a top-level `CardDefinition` field to a `Static` ability variant processed through the layer system.

**How:** The casting system currently scans raw `CardDefinition` fields for cost modifiers. Instead, it should:

1. Iterate over battlefield permanents
2. Resolve each permanent's abilities through `calculate_characteristics`
3. Extract cost modifiers from the resolved ability set
4. Sum and apply them to the spell's cost

Thalia's definition becomes:

```rust
abilities: vec![
    AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
    AbilityDefinition::Static {
        continuous_effect: ContinuousEffect {
            layer: EffectLayer::Ability, // Layer 6 — removed by Humility
            modification: LayerModification::SpellCostModifier(SpellCostModifier {
                change: 1,
                filter: SpellCostFilter::NonCreature,
                scope: CostModifierScope::AllPlayers,
            }),
        },
    },
],
spell_cost_modifiers: vec![], // No longer used for this purpose
```

**Cost:** Casting becomes slightly more expensive (layer resolution per permanent per cast). Casting is not a hot path, so this is acceptable.

**Risk:** Low. This is a refactor of the data model and one call site.

### Fix 2: Resolution Suspension for Player Choices

**What:** Allow `resolve_top_of_stack` to suspend when it encounters a choice point during resolution, request player input, and resume.

**How:** Introduce a result type that models suspension:

```rust
enum ResolutionResult {
    Complete(GameState, Vec<GameEvent>),
    NeedsChoice {
        state: GameState,
        pending: PendingResolution,
        choice_request: ChoiceRequest,
    },
}
```

`PendingResolution` captures the remaining effect tree and the context needed to continue. The external layer sends `ChoiceRequest` to the appropriate player, receives a response, and feeds it back as `Command::MakeChoice`. Resolution resumes from the saved state.

This is essentially modeling the effect execution as a coroutine. Rust doesn't have native coroutines, but explicit suspension via `PendingResolution` is idiomatic. The immutable state design makes this easier than it would be in a mutable engine — the state at the choice point is a snapshot that can be resumed cleanly.

**What this unblocks:**

- All "target opponent chooses" effects
- Modal spells with resolution-time mode selection
- Replacement effect ordering (CR 614.5) — same pattern, different trigger
- Eventually, Panglacial Wurm (mid-search casting is a special case of mid-resolution choice)

**Cost:** Threading `ChoiceRequest` variants through the effect system. Every effect that can require a choice during resolution needs to be identified and handled. This is real work but additive — it doesn't break anything that already works.

**Risk:** Medium. The `PendingResolution` struct needs to capture enough context to resume correctly, and the interaction between suspension and trigger checking needs careful design.

### Fix 3: LKI Snapshots on Zone Changes

**What:** Whenever an object changes zones, snapshot its layer-resolved characteristics and store them for later reference.

**How:** Add to `GameState`:

```rust
pub last_known_info: HashMap<ObjectId, LKISnapshot>,
```

Where `LKISnapshot` contains the fully-resolved `Characteristics` (after all layer effects) plus zone, controller, and any other relevant state.

The snapshot is taken *before* the zone change is applied:

```rust
fn move_object_to_zone(state: &mut GameState, object_id: ObjectId, new_zone: ZoneId) {
    // Snapshot LKI before the move
    if let Some(chars) = calculate_characteristics(state, object_id) {
        let obj = state.objects.get(&object_id).unwrap();
        state.last_known_info.insert(object_id, LKISnapshot {
            characteristics: chars,
            zone: obj.zone,
            controller: obj.controller,
            counters: obj.counters.clone(),
        });
    }
    // Now perform the zone change
    // ...
}
```

Any effect that references an object no longer in its expected zone falls back to LKI. This is the standard approach and matches what MTGO does internally.

**Cost:** One `calculate_characteristics` call per zone change. Zone changes are infrequent relative to other operations.

**Risk:** Low. Well-understood pattern.

### Fix 4: Compositional Trigger Conditions

**What:** Decompose monolithic trigger condition variants into composable primitives.

**How:** Instead of:

```rust
TriggerCondition::WheneverYouCastSpell {
    spell_type_filter: Some(vec![CardType::Creature]),
    during_opponent_turn: false,
    noncreature_only: false,
}
```

Move to:

```rust
TriggerCondition {
    event: TriggerEvent::SpellCast,
    filter: TriggerFilter::And(vec![
        TriggerFilter::ControlledByYou,
        TriggerFilter::HasType(CardType::Creature),
    ]),
    first_time_per: None,
    zone_from: None,
}
```

This makes "whenever you cast a creature spell," "whenever you cast a spell from anywhere other than your hand," and "whenever you cast a creature spell for the first time each turn" all compositions of the same primitives.

For "first time each turn" tracking, add auxiliary state:

```rust
pub trigger_counts: HashMap<(ObjectId, usize), u32>, // (source, trigger_index) → count this turn
```

Reset at turn boundaries. The trigger system checks this map when `first_time_per` is set.

**Cost:** Refactoring existing trigger conditions into the new compositional format. The existing card definitions need migration, but the behavior is equivalent.

**Risk:** Low-medium. The migration is mechanical but touches many card definitions.

### Fix 5: Turn Control and Step Skipping

**What:** Add Mindslaver-style turn control and Necropotence-style step skipping as features of the turn structure.

**How:**

For turn control:

```rust
// In GameState or TurnState
pub turn_controller_override: Option<(PlayerId, PlayerId)>, // (controller, controlled)
```

Command processing checks this field. When player B is controlled by player A, all of player B's `Command` variants are routed to player A for decision-making. Player B's hand is revealed to player A. This doesn't require architectural changes — it's a modifier on the Command routing layer.

For step skipping:

```rust
// In TurnState
pub skip_steps: HashSet<(PlayerId, Step)>,
```

When the turn structure would advance to a step in the skip set, it advances past it. Effects like Necropotence register a skip. Remove entries at end of turn or as specified by the effect.

**Cost:** Small, well-scoped additions to `TurnState` and Command routing.

**Risk:** Low.

### Fix 6: DSL Escape Hatch (Deferred)

**What:** When the DSL can no longer express a card's behavior through composition of existing primitives, provide a contained escape hatch.

**When:** Not needed now. Target: when card coverage exceeds ~2,500 and specific cards are identified that cannot be expressed.

**How (recommended approach):** An `Effect::Custom(CardId)` variant that dispatches to a registered Rust function:

```rust
type CustomEffectFn = fn(&mut GameState, &mut EffectContext) -> Vec<GameEvent>;

// In a registry, not in the card definition
custom_effects: HashMap<CardId, CustomEffectFn>,
```

The custom functions operate on `GameState` through the same APIs as the generic effect system, so they compose with triggers, layers, and SBAs correctly. This is philosophically impure (it's card-specific code), but it's contained. An estimated 50–100 cards out of the full card pool would need it. The rest of the architecture stays generic.

**Alternative (not recommended unless community card authoring is planned):** Embed a small scripting language (Lua). This is more powerful but adds binding complexity and loses Rust's type safety on card definitions.

**Risk:** Low if contained to a small number of cards. The key discipline is that custom functions must use the same `GameState` mutation APIs as the generic system — they cannot bypass triggers, layers, or SBAs.

---

## Part IV: Priority Roadmap

| Priority | Fix | Rationale |
|----------|-----|-----------|
| **P0 — Now** | Fix 1: Cost modifiers into ability system | Active rules bug. Produces incorrect results with Humility, Dress Down, Overwhelming Splendor, and any ability-removing effect interacting with cost modifiers. |
| **P1 — Before alpha** | Fix 2: Resolution suspension | Unblocks the largest class of currently-unimplementable cards. Many Commander staples require choices during resolution. |
| **P1 — Before alpha** | Fix 3: LKI snapshots | Load-bearing for correctness across zone-change interactions. Affects any card that references an object after it has moved zones. |
| **P2 — During alpha** | Fix 4: Compositional triggers | Reduces per-card maintenance burden and accommodates new trigger patterns without engine changes. Can be migrated incrementally. |
| **P2 — During alpha** | Fix 5: Turn control and step skipping | Feature gaps that block specific popular Commander cards (Mindslaver, Emrakul the Promised End, Necropotence, Stasis). |
| **P3 — Post-alpha** | Fix 6: DSL escape hatch | Only needed when specific cards are identified that the DSL cannot express. Design it when you need it, not before. |

---

## Closing Assessment

The architecture is strong. The issues identified here are elaborations and corrections within a well-designed system, not symptoms of a flawed foundation. The most important signal is that every fix described above is additive — none of them require tearing out and replacing existing systems. That's the hallmark of an architecture that got the decomposition right.

The engine's six-system model (state, commands, card definitions, effects, triggers, layers) is the correct decomposition for MTG. The work ahead is making each system more expressive and more interactive, not replacing any of them.
