# How the MTG Rules Engine Works

> **Audience**: narrative walkthrough for new contributors and external readers.
> This document is a concrete explanatory tour of the engine: what the core idea
> is, what the six key systems do, and how they compose on a worked example.
> For the design-decision and testing-strategy reference used day-to-day during
> engine development, see `docs/mtg-engine-architecture.md`. The two documents
> are intentionally kept separate — this one is the "how it works" orientation,
> the architecture doc is the "why we chose this" reference.

This document explains the architecture of a Magic: The Gathering rules engine built in
Rust, targeting Commander format (4-player multiplayer). It covers the high-level design,
the key systems that make it work, and walks through a concrete example showing how
multiple cards interact correctly without any card-specific code.

---

## The Core Idea

Magic has ~27,000 unique cards, each with different abilities. Those abilities interact
in combinatorial ways — any card can appear alongside any other card, and the rules must
handle every combination correctly. Building card-specific interaction code is impossible
at scale.

Instead, this engine separates **what cards do** from **how the game processes it**:

- **Card definitions** are pure data. They declare abilities using a structured DSL
  (domain-specific language) of effects, triggers, and conditions.
- **The rules engine** is a set of generic systems that process any card definition
  correctly. It doesn't know what "Lightning Bolt" is — it knows how to deal damage,
  check triggers, resolve the stack, and apply state-based actions.

This separation means adding a new card is just adding data. The engine's behavior
doesn't change. A card definition for a new creature with a triggered ability works
correctly alongside 1,700 other cards because the trigger system, the stack, and the
layer system are all generic.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    External Layer                        │
│         Network server, UI, replay viewer               │
│              (not part of the engine)                    │
└─────────────────────┬───────────────────────────────────┘
                      │ Commands in, Events out
┌─────────────────────▼───────────────────────────────────┐
│                                                         │
│                    RULES ENGINE                         │
│                                                         │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────┐  │
│  │   Command    │  │   Effect     │  │    Layer      │  │
│  │  Processing  │  │  Execution   │  │   System      │  │
│  │  (engine.rs) │  │ (effects/)   │  │  (layers.rs)  │  │
│  └──────┬───────┘  └──────┬───────┘  └───────┬───────┘  │
│         │                 │                   │          │
│  ┌──────▼───────┐  ┌──────▼───────┐  ┌───────▼───────┐  │
│  │   Trigger    │  │    Stack     │  │  State-Based  │  │
│  │  Detection   │  │  Resolution  │  │   Actions     │  │
│  │(abilities.rs)│  │(resolution.rs│  │   (sba.rs)    │  │
│  └──────────────┘  └──────────────┘  └───────────────┘  │
│                                                         │
│  ┌─────────────────────────────────────────────────────┐ │
│  │              Immutable Game State                   │ │
│  │  players, objects, stack, continuous effects,       │ │
│  │  triggers, replacement effects, zones               │ │
│  │         (im-rs persistent data structures)          │ │
│  └─────────────────────────────────────────────────────┘ │
│                                                         │
│  ┌─────────────────────────────────────────────────────┐ │
│  │          Card Registry (Arc<CardRegistry>)          │ │
│  │    1,452 CardDefinitions loaded at game start       │ │
│  │         O(1) lookup by CardId (HashMap)             │ │
│  └─────────────────────────────────────────────────────┘ │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**Key invariants:**

1. **The engine is a pure library.** No I/O, no network, no filesystem, no async. It
   takes commands in and emits events out.
2. **Game state is immutable.** State transitions produce new states via persistent data
   structures (`im-rs`). Old states are retained for undo and replay.
3. **All player actions are Commands.** There is no way to change game state except
   through the `Command` enum.
4. **All state changes are Events.** The engine emits `GameEvent` values describing what
   happened. The network layer broadcasts these; the UI consumes them.

---

## The Six Key Systems

### 1. Game State — The World

`GameState` holds the complete, serializable state of a game in progress:

```rust
pub struct GameState {
    pub turn: TurnState,                              // Phase, step, active player, priority
    pub players: OrdMap<PlayerId, PlayerState>,        // Life, poison, mana, hand size
    pub objects: OrdMap<ObjectId, GameObject>,          // Every card/token in every zone
    pub zones: OrdMap<ZoneId, Zone>,                   // Libraries, hands, graveyards, etc.
    pub stack_objects: Vector<StackObject>,             // The stack (spells & abilities)
    pub continuous_effects: Vector<ContinuousEffect>,  // Layer system effects (CR 611)
    pub replacement_effects: Vector<ReplacementEffect>,// Inline event modifiers (CR 614)
    pub pending_triggers: Vector<PendingTrigger>,      // Triggers waiting for the stack
    pub trigger_doublers: Vector<TriggerDoubler>,      // Panharmonicon-style effects
    pub etb_suppressors: Vector<ETBSuppressor>,        // Torpor Orb-style effects
    pub card_registry: Arc<CardRegistry>,              // All card definitions (shared)
    // ... additional fields for delayed triggers, prevention, commander choices, etc.
}
```

Every field uses `im-rs` persistent data structures (`OrdMap`, `Vector`), which support
structural sharing — cloning a game state is O(1) because unchanged subtrees are shared.
This makes undo, replay, and branching cheap.

The `card_registry` is `Arc`-wrapped and shared across all state snapshots. It contains
every `CardDefinition` needed for the game, loaded at startup and never modified.

### 2. Command Processing — The Entry Point

Every player action enters through a single function:

```rust
pub fn process_command(
    state: GameState,
    command: Command,
) -> Result<(GameState, Vec<GameEvent>), GameStateError>
```

It takes ownership of the current state, processes the command, and returns a new state
plus a list of events. The old state is untouched.

The `Command` enum covers every legal player action:

```rust
enum Command {
    PassPriority { player },
    CastSpell { player, card, targets, kicker_times, alt_cost, modes_chosen, x_value, ... },
    ActivateAbility { player, source, ability_index, targets, ... },
    PlayLand { player, card },
    TapForMana { player, source, ability_index },
    DeclareAttackers { player, attackers },
    DeclareBlockers { player, blockers },
    Concede { player },
    // ... ~26 total variants for mulligan, commander, forecast, bloodrush, etc.
}
```

After every command that changes the game, `process_command` calls
`check_and_flush_triggers`:

```rust
fn check_and_flush_triggers(state: &mut GameState, events: &mut Vec<GameEvent>) {
    // CR 603.3: detect triggered abilities from the events that just happened
    let new_triggers = abilities::check_triggers(state, events);
    for t in new_triggers {
        state.pending_triggers.push_back(t);
    }
    // Convert pending triggers into stack objects (APNAP order)
    let trigger_events = abilities::flush_pending_triggers(state);
    events.extend(trigger_events);
}
```

This is the critical integration point. Every spell cast, every land played, every ability
activated — the engine immediately checks whether anything triggered, and if so, puts
those triggers on the stack. This is how card interactions emerge without card-specific code.

When all players pass priority in succession:

```rust
fn handle_all_passed(state: &mut GameState) -> Result<Vec<GameEvent>, GameStateError> {
    if !state.stack_objects.is_empty() {
        // CR 608.1: resolve the top of the stack
        resolution::resolve_top_of_stack(state)
    } else {
        // Stack empty — advance to next step/phase/turn
        turn_structure::advance_step(state)
    }
}
```

### 3. Card Definitions — Pure Data

A `CardDefinition` describes a card entirely in terms the engine can execute:

```rust
pub struct CardDefinition {
    pub card_id: CardId,                  // Stable identity (Scryfall oracle_id)
    pub name: String,                     // Display name
    pub mana_cost: Option<ManaCost>,      // Printed mana cost
    pub types: TypeLine,                  // Supertypes, card types, subtypes
    pub oracle_text: String,              // For display only — behavior is in abilities
    pub abilities: Vec<AbilityDefinition>,// All abilities, in oracle text order
    pub power: Option<i32>,               // Creature P/T (None for non-creatures)
    pub toughness: Option<i32>,
    pub spell_cost_modifiers: Vec<SpellCostModifier>,  // "Spells cost {1} more/less"
    pub back_face: Option<CardFace>,      // Double-faced cards (Transform, Disturb)
    // ... additional optional fields with defaults
}
```

The key field is `abilities: Vec<AbilityDefinition>`. This is where all card behavior
lives. `AbilityDefinition` is an enum with variants for every type of Magic ability:

```rust
enum AbilityDefinition {
    // "When/Whenever/At [event], [effect]" (CR 603)
    Triggered {
        trigger_condition: TriggerCondition,
        effect: Effect,
        intervening_if: Option<Condition>,
        targets: Vec<TargetRequirement>,
    },

    // "[Cost]: [Effect]" (CR 602)
    Activated {
        cost: ActivationCost,
        effect: Effect,
        targets: Vec<TargetRequirement>,
        sorcery_speed: bool,
        activation_condition: Option<Condition>,
    },

    // Continuous effect on the battlefield (CR 604)
    Static { continuous_effect: ContinuousEffect },

    // Quick keyword presence (Flying, Haste, Lifelink, etc.) — ~158 variants
    Keyword(KeywordAbility),

    // Spell resolution effect (instants/sorceries)
    Spell {
        effect: Effect,
        targets: Vec<TargetRequirement>,
        modes: Option<Vec<ModeSelection>>,
        cant_be_countered: bool,
    },

    // Event modification that doesn't use the stack (CR 614-615)
    Replacement {
        trigger: ReplacementTrigger,
        modification: ReplacementModification,
        is_self: bool,
        unless_condition: Option<Condition>,
    },

    // ... specialized variants for cycling, kicker, evoke, partner, etc.
}
```

#### Real Card Examples

**Lightning Bolt** — the simplest possible spell:

```rust
// Lightning Bolt — {R}, Instant
// "Lightning Bolt deals 3 damage to any target."
pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lightning-bolt"),
        name: "Lightning Bolt".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Lightning Bolt deals 3 damage to any target.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DealDamage {
                target: EffectTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(3),
            },
            targets: vec![TargetRequirement::TargetAny],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
```

The card declares: "I'm a spell that deals 3 damage to target 0, and target 0 must be
any legal target." The engine handles targeting validation, stack placement, priority
passing, resolution, damage prevention, and trigger checking — all generically.

**Beast Whisperer** — a triggered ability:

```rust
// Beast Whisperer — {2}{G}{G}, Creature — Elf Druid 2/3
// "Whenever you cast a creature spell, draw a card."
pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("beast-whisperer"),
        name: "Beast Whisperer".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: creature_types(&["Elf", "Druid"]),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverYouCastSpell {
                during_opponent_turn: false,
                spell_type_filter: Some(vec![CardType::Creature]),
                noncreature_only: false,
            },
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            intervening_if: None,
            targets: vec![],
        }],
        ..Default::default()
    }
}
```

Beast Whisperer declares a trigger condition (`WheneverYouCastSpell` filtered to
creatures) and an effect (`DrawCards`). When any creature spell is cast, the trigger
detection system pattern-matches this condition against the "spell cast" event, queues
it, and puts it on the stack. The controller draws a card when it resolves. Beast
Whisperer doesn't know what creature was cast, and doesn't need to.

**Impact Tremors** — composing effects:

```rust
// Impact Tremors — {1}{R}, Enchantment
// "Whenever a creature you control enters, this enchantment deals 1 damage
//  to each opponent."
pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("impact-tremors"),
        name: "Impact Tremors".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        abilities: vec![AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                filter: Some(TargetFilter {
                    controller: TargetController::You,
                    ..Default::default()
                }),
            },
            effect: Effect::ForEach {
                over: ForEachTarget::EachOpponent,
                effect: Box::new(Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(1),
                }),
            },
            intervening_if: None,
            targets: vec![],
        }],
        ..Default::default()
    }
}
```

The `ForEach` combinator iterates over each opponent and deals 1 damage to each. In a
4-player Commander game, this automatically deals 1 damage to all three opponents. The
card doesn't know how many opponents there are — `ForEach::EachOpponent` handles it.

**Swords to Plowshares** — sequenced effects with cross-references:

```rust
// Swords to Plowshares — {W}, Instant
// "Exile target creature. Its controller gains life equal to its power."
pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("swords-to-plowshares"),
        name: "Swords to Plowshares".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::ExileObject {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                Effect::GainLife {
                    player: PlayerTarget::ControllerOf(
                        Box::new(EffectTarget::DeclaredTarget { index: 0 })
                    ),
                    amount: EffectAmount::PowerOf(
                        EffectTarget::DeclaredTarget { index: 0 }
                    ),
                },
            ]),
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
```

`Effect::Sequence` executes effects in order. After exiling the creature, the engine
uses last-known information (LKI) to determine the creature's power and controller — the
creature is in exile by the time life is gained, but `EffectTarget::DeclaredTarget`
tracks the object across zone changes via target remapping.

**Thalia, Guardian of Thraben** — a static cost modifier:

```rust
// Thalia, Guardian of Thraben — {1}{W}, Legendary Creature — Human Soldier 2/1
// "First strike. Noncreature spells cost {1} more to cast."
pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thalia-guardian-of-thraben"),
        name: "Thalia, Guardian of Thraben".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Soldier"],
        ),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
        ],
        spell_cost_modifiers: vec![SpellCostModifier {
            change: 1,
            filter: SpellCostFilter::NonCreature,
            scope: CostModifierScope::AllPlayers,
            eminence: false,
            exclude_self: false,
        }],
        ..Default::default()
    }
}
```

Thalia's tax isn't an ability that triggers or activates — it's a `spell_cost_modifier`
that the casting system checks when any player casts a spell. The engine scans all
battlefield permanents for applicable cost modifiers, sums them up, and adjusts the cost.
If two Thalias are on the battlefield, noncreature spells cost {2} more. No special code
needed — the modifier system handles stacking automatically.

### 4. Effect Execution — Walking the Tree

Effects are a recursive tree. The `execute_effect` function walks it:

```rust
pub fn execute_effect(
    state: &mut GameState,
    effect: &Effect,
    ctx: &mut EffectContext,   // controller, source, targets, modifiers
) -> Vec<GameEvent>
```

The `Effect` enum has ~60 variants organized by category:

| Category | Examples |
|----------|----------|
| Damage & Life | `DealDamage`, `GainLife`, `LoseLife`, `DrainLife` |
| Cards | `DrawCards`, `DiscardCards`, `MillCards` |
| Permanents | `CreateToken`, `DestroyPermanent`, `DestroyAll`, `ExileObject` |
| Counters | `AddCounter`, `RemoveCounter`, `Bolster`, `Amass` |
| Zone Changes | `MoveZone`, `SearchLibrary`, `Scry`, `Surveil` |
| Continuous | `ApplyContinuousEffect` (registers a layer-system effect) |
| **Combinators** | `Conditional`, `ForEach`, `Sequence`, `Choose`, `Repeat` |

The combinators are what make the DSL expressive. A single card can compose any
combination:

```rust
// "If this spell was kicked, destroy target creature.
//  Otherwise, deal 3 damage to target creature."
Effect::Conditional {
    condition: Condition::WasKicked,
    if_true: Box::new(Effect::DestroyPermanent {
        target: EffectTarget::DeclaredTarget { index: 0 },
        ..
    }),
    if_false: Box::new(Effect::DealDamage {
        target: EffectTarget::DeclaredTarget { index: 0 },
        amount: EffectAmount::Fixed(3),
    }),
}
```

Each effect variant maps to a concrete game action. Here's a simplified view of
`DealDamage`:

```rust
Effect::DealDamage { target, amount } => {
    let dmg = resolve_amount(state, amount, ctx).max(0) as u32;
    let targets = resolve_effect_target_list(state, target, ctx);
    for resolved in targets {
        match resolved {
            ResolvedTarget::Player(p) => {
                // Apply damage doubling (e.g., Furnace of Rath)
                let dmg = apply_damage_doubling(state, ctx.source, dmg, ..);
                // Apply damage prevention (e.g., protection, Fog)
                let dmg = apply_damage_prevention(state, ctx.source, &target, dmg);
                // Check for infect (poison counters instead of life loss)
                if source_has_infect { add_poison_counters(..) }
                else { reduce_life(..) }
                events.push(GameEvent::DamageDealt { .. });
            }
            ResolvedTarget::Object(id) => {
                // Mark damage on the permanent (for SBA checking later)
                obj.damage_marked += dmg;
                events.push(GameEvent::DamageDealt { .. });
            }
        }
    }
}
```

Notice how damage execution weaves in replacement effects (doubling, prevention) and
keyword checks (infect) without the card definition knowing about any of them. Lightning
Bolt just says "deal 3 damage." Furnace of Rath's doubling, a protection effect's
prevention, and infect's poison counters are all handled by the engine during execution.

### 5. Trigger Detection — Pattern Matching Events

After every state change, the engine scans all objects on the battlefield to check if
anything triggered:

```rust
pub fn check_triggers(state: &GameState, events: &[GameEvent]) -> Vec<PendingTrigger> {
    let mut triggers = Vec::new();
    for event in events {
        match event {
            GameEvent::PermanentEnteredBattlefield { object_id, .. } => {
                // Check SelfEntersBattlefield on the entering permanent
                collect_triggers_for_event(state, &mut triggers,
                    TriggerEvent::SelfEntersBattlefield, Some(*object_id), ..);
                // Check AnyPermanentEntersBattlefield on ALL permanents
                collect_triggers_for_event(state, &mut triggers,
                    TriggerEvent::AnyPermanentEntersBattlefield, None, ..);
            }
            GameEvent::CreatureDied { object_id, .. } => { /* similarly */ }
            GameEvent::SpellCast { .. } => { /* similarly */ }
            // ... for every event type
        }
    }
    triggers
}
```

The inner `collect_triggers_for_event` function scans objects and matches their trigger
conditions:

```rust
fn collect_triggers_for_event(state, triggers, event_type, only_object, entering_object) {
    // Determine which objects to scan
    let objects = if let Some(id) = only_object {
        vec![id]                    // Just one object (self-triggers)
    } else {
        all_battlefield_objects()   // Everything (other-triggers)
    };

    for obj_id in objects {
        // CR 708.3: face-down permanents have no triggered abilities
        if obj.face_down { continue; }

        // CR 613.1f (Layer 6): use layer-resolved abilities so that
        // ability-removing effects (Humility) suppress triggers
        let resolved = calculate_characteristics(state, obj_id);

        for trigger_def in resolved.triggered_abilities {
            if trigger_def.trigger_on == event_type {
                // Match! Check intervening-if condition (CR 603.4)
                if check_condition(state, &trigger_def.intervening_if) {
                    triggers.push(PendingTrigger {
                        source: obj_id,
                        controller: obj.controller,
                        // ... event context
                    });
                }
            }
        }
    }
}
```

This is the core of how card interactions work. When a creature enters the battlefield:

1. The engine emits `PermanentEnteredBattlefield`
2. `check_triggers` scans every permanent on the battlefield
3. Any permanent with a matching trigger condition gets a `PendingTrigger` queued
4. Pending triggers are flushed to the stack in APNAP order (active player first)
5. Each trigger resolves individually with full priority passing

Impact Tremors, Beast Whisperer, Soul Warden, Panharmonicon — they all work through
this same system. No card knows about any other card. They just declare conditions, and
the engine matches them.

### 6. The Layer System — Resolving Static Effects

When any system needs to know a permanent's actual characteristics (power, toughness,
abilities, types, colors), it calls `calculate_characteristics`. This function applies
all active continuous effects in strict layer order per the Comprehensive Rules:

```rust
pub fn calculate_characteristics(
    state: &GameState,
    object_id: ObjectId,
) -> Option<Characteristics> {
    let obj = state.objects.get(&object_id)?;
    let mut chars = obj.characteristics.clone();  // Start with base (printed) values

    // Pre-layer: handle face-down (Morph = 2/2 colorless), DFC back faces, etc.
    if obj.is_transformed {
        // Use back face characteristics from CardDefinition
    }
    if obj.face_down {
        // Override to 2/2 colorless creature with no abilities (CR 708.2)
    }

    // Collect all active continuous effects
    let active_effects: Vec<&ContinuousEffect> = state.continuous_effects
        .iter()
        .filter(|e| is_effect_active(state, e))
        .collect();

    // Apply layers in strict CR 613.1 order
    let layers = [
        EffectLayer::Copy,       // Layer 1: copy effects
        EffectLayer::Control,    // Layer 2: control changes
        EffectLayer::Text,       // Layer 3: text changes
        EffectLayer::TypeChange, // Layer 4: type changes
        EffectLayer::ColorChange,// Layer 5: color changes
        EffectLayer::Ability,    // Layer 6: ability add/remove
        EffectLayer::PtCda,      // Layer 7a: P/T from CDAs
        EffectLayer::PtSet,      // Layer 7b: P/T setting
        EffectLayer::PtModify,   // Layer 7c: P/T modifying (+1/+1, anthems)
        EffectLayer::PtSwitch,   // Layer 7d: P/T switching
    ];

    for layer in layers {
        // Within each layer: dependency sort, then timestamp order (CR 613.7-8)
        let effects_in_layer = active_effects
            .iter()
            .filter(|e| e.layer == layer && e.applies_to(object_id));
        for effect in sorted_by_dependency_then_timestamp(effects_in_layer) {
            apply_layer_modification(&mut chars, effect);
        }
    }

    // Apply +1/+1 and -1/-1 counters (Layer 7c)
    chars.power += obj.counters.p1p1 - obj.counters.m1m1;
    chars.toughness += obj.counters.p1p1 - obj.counters.m1m1;

    Some(chars)
}
```

This function is called dynamically — combat checks it, SBAs check it, trigger detection
checks it. Whenever someone asks "does this creature have flying?" or "what's its
toughness?", the layer system computes the answer by applying every continuous effect in
the correct order.

This is how a card like Humility ("All creatures lose all abilities and are 1/1") works.
It registers a continuous effect at Layer 6 (remove abilities) and Layer 7b (set P/T
to 1/1). The layer system applies it to every creature, and all other systems see the
result. Trigger detection uses layer-resolved abilities, so Humility automatically
suppresses triggered abilities. No special case needed.

### 7. State-Based Actions — The Cleanup Loop

After every priority pass, the engine checks state-based actions in a fixed-point loop:

```rust
pub fn check_and_apply_sbas(state: &mut GameState) -> Vec<GameEvent> {
    loop {
        let events = apply_sbas_once(state);
        if events.is_empty() { break; }    // Fixed point — nothing happened

        // CR 704.3: check triggers from this pass's events
        // before the next pass can remove objects
        let triggers = check_triggers(state, &events);
        for t in triggers {
            state.pending_triggers.push_back(t);
        }
    }
}
```

Each SBA pass snapshots every battlefield permanent's characteristics through the layer
system, then checks all rules simultaneously:

```rust
fn apply_sbas_once(state: &mut GameState) -> Vec<GameEvent> {
    // Build characteristics snapshot for ALL battlefield objects
    let chars_map: HashMap<ObjectId, Characteristics> = battlefield_ids
        .iter()
        .filter_map(|&id| {
            let chars = calculate_characteristics(state, id)?;
            Some((id, chars))
        })
        .collect();

    let mut events = Vec::new();

    // All SBAs checked simultaneously (CR 704.3):
    events.extend(check_ascend(state, &chars_map));       // CR 702.131b
    events.extend(check_player_sbas(state));               // CR 704.5a: 0 life
    events.extend(check_creature_sbas(state, &chars_map)); // CR 704.5f-h: lethal damage, 0 toughness
    events.extend(check_planeswalker_sbas(state, &chars_map)); // CR 704.5i: 0 loyalty
    events.extend(check_legendary_rule(state));            // CR 704.5j: duplicate legends
    events.extend(check_aura_sbas(state));                 // CR 704.5m: illegal attachment
    events.extend(check_equipment_sbas(state, &chars_map));// CR 704.5n: illegal equip
    events.extend(check_counter_annihilation(state));      // CR 704.5q: +1/+1 and -1/-1 cancel
    events.extend(check_commander_zone_return(state));     // CR 903.9a
    events.extend(check_dungeon_completion(state));        // CR 704.5t
    // ... more SBA categories

    events
}
```

The fixed-point loop is critical: SBAs can cause more SBAs. A creature dying to lethal
damage might cause a death trigger that creates a token; the token entering might trigger
Ascend. The loop keeps running until nothing happens.

Between passes, triggers from SBA events are queued (but not flushed to the stack until
the loop finishes). This ensures "when this creature dies" triggers fire correctly even
though the creature is removed from the battlefield during the SBA pass.

---

## Putting It All Together: A Concrete Interaction

Here's a realistic Commander scenario that exercises all six systems simultaneously.

### Setup

Four-player Commander game. The board state:

- **Player 1** controls:
  - **Impact Tremors** (enchantment: "Whenever a creature you control enters, deal 1
    damage to each opponent")
  - **Beast Whisperer** (creature: "Whenever you cast a creature spell, draw a card")
  - **Thalia, Guardian of Thraben** (creature: "Noncreature spells cost {1} more")

- **Players 2, 3, 4** are at 40 life each

Player 1 casts **Lightning Bolt** targeting Player 2.

### What Happens

**Step 1: Command Processing**

Player 1 sends:
```
Command::CastSpell {
    player: PlayerId(1),
    card: lightning_bolt_id,
    targets: [SpellTarget::Player(PlayerId(2))],
    ...
}
```

`process_command` dispatches to `handle_cast_spell`.

**Step 2: Cost Calculation**

The casting system scans all battlefield permanents for `spell_cost_modifiers`. It finds
Thalia's modifier:

```rust
SpellCostModifier {
    change: 1,
    filter: SpellCostFilter::NonCreature,   // Lightning Bolt is an instant
    scope: CostModifierScope::AllPlayers,
}
```

Lightning Bolt normally costs {R}. With Thalia's tax, it costs {1}{R}. Player 1 must pay
the extra generic mana.

**Step 3: Stack Placement**

After cost payment, Lightning Bolt goes on the stack as a `StackObject`. The engine emits
`GameEvent::SpellCast`.

**Step 4: Trigger Detection**

`check_and_flush_triggers` runs. It processes the `SpellCast` event:

- Beast Whisperer's trigger condition is `WheneverYouCastSpell { spell_type_filter:
  Some([Creature]) }`. Lightning Bolt is an instant, not a creature. **No match.**

No triggers fire. Priority passes to all players.

**Step 5: Resolution**

All four players pass priority. `handle_all_passed` calls
`resolve_top_of_stack`.

The engine pops Lightning Bolt from the stack, checks that Player 2 is still a legal
target, then executes:

```rust
Effect::DealDamage {
    target: EffectTarget::DeclaredTarget { index: 0 },  // Player 2
    amount: EffectAmount::Fixed(3),
}
```

`execute_effect` resolves the target to Player 2, checks for damage prevention (none),
and deals 3 damage. Player 2 is now at 37 life. The engine emits
`GameEvent::DamageDealt`.

Lightning Bolt goes to the graveyard. `GameEvent::SpellResolved` is emitted.

**Step 6: Post-Resolution SBAs + Triggers**

`check_and_apply_sbas` runs. Player 2 is at 37 life — no SBA. All creatures have
positive toughness. No legendary duplicates. Nothing happens.

**Step 7: Priority Reset**

Priority resets to Player 1 (active player). The game continues.

---

### Now Player 1 Casts a Creature

Player 1 casts a **Llanowar Elves** (1/1 creature).

**Cost Calculation:** Thalia's modifier is `NonCreature` — Llanowar Elves is a creature,
so no tax. Cost is just {G}.

**Stack + Trigger Detection:** `SpellCast` event fires. This time, Beast Whisperer's
`WheneverYouCastSpell { spell_type_filter: Some([Creature]) }` **matches**. A
`PendingTrigger` is created and flushed to the stack above Llanowar Elves.

**Stack state:**
```
Top:  Beast Whisperer's trigger (draw a card)
      Llanowar Elves (creature spell)
```

All players pass. Beast Whisperer's trigger resolves — Player 1 draws a card.

All players pass again. Llanowar Elves resolves — it enters the battlefield. The engine
emits `PermanentEnteredBattlefield`.

**Trigger Detection:** `check_triggers` scans all permanents:
- Impact Tremors has `WheneverCreatureEntersBattlefield { filter: controller == You }`.
  Llanowar Elves is a creature controlled by Player 1. **Match!**

Impact Tremors' trigger goes on the stack. All players pass. It resolves:

```rust
Effect::ForEach {
    over: ForEachTarget::EachOpponent,        // Players 2, 3, 4
    effect: Box::new(Effect::DealDamage {
        target: EffectTarget::DeclaredTarget { index: 0 },
        amount: EffectAmount::Fixed(1),
    }),
}
```

The engine iterates over each opponent and deals 1 damage. Player 2 goes to 36, Player 3
to 39, Player 4 to 39. Three `DamageDealt` events are emitted.

**SBAs:** All players still alive. Llanowar Elves has 1 toughness with 0 damage. Nothing
happens.

---

### Why This Works Without Card-Specific Code

At no point did the engine contain logic like "if Impact Tremors is on the battlefield
and a creature enters, deal damage." Instead:

1. **Impact Tremors' definition** declares a trigger condition and an effect
2. **The trigger system** pattern-matches the condition against the ETB event
3. **The stack** handles ordering and priority
4. **The effect engine** executes `ForEach` over opponents and `DealDamage` to each
5. **SBAs** check if anyone died

If Player 1 also controlled a **Soul Warden** ("Whenever another creature enters, gain 1
life"), that card's trigger condition would also match the same ETB event. Both triggers
would be queued in the same pass. Player 1 would choose their order on the stack (since
they're the same controller). Both would resolve. The system scales to any number of
cards with matching trigger conditions — they all just declare conditions, and the
engine matches them all in one pass.

If Humility ("All creatures lose all abilities and are 1/1") were on the battlefield,
`calculate_characteristics` would strip Beast Whisperer's triggered ability at Layer 6.
The trigger detection system uses layer-resolved abilities, so Beast Whisperer's trigger
would never fire. Thalia's `spell_cost_modifier` lives on the `CardDefinition` (not as
an ability), so it would still apply even under Humility. All of this falls out of the
architecture — no special interaction code.

---

## Summary

| System | What It Does | Why It Matters |
|--------|-------------|----------------|
| **Game State** | Immutable snapshot of the entire game | Enables undo, replay, branching |
| **Command Processing** | Single entry point for all player actions | Every action goes through the same pipeline |
| **Card Definitions** | Declarative data describing card behavior | Adding cards = adding data, not code |
| **Effect Execution** | Recursive tree-walker for effect resolution | Composable effects from ~60 primitives |
| **Trigger Detection** | Pattern-matching events against all permanents | Card interactions emerge from generic matching |
| **Layer System** | Ordered application of continuous effects | Static effects compose correctly per CR 613 |
| **State-Based Actions** | Fixed-point loop checking game rules | Deaths, legend rule, etc. handled uniformly |

The engine processes ~2,300 tests covering these systems. Card definitions are pure data
— currently 1,452 cards defined, with a target of 1,743 for the alpha release. Every
card works correctly alongside every other card because the rules engine is generic. The
combinatorial complexity lives in the engine's six systems, not in the card definitions.
