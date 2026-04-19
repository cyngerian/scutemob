---
name: bulk-card-author
description: |
  Use this agent to author a batch of CardDefinition files from an authoring plan session.
  Reads session data from _authoring_plan.json, looks up oracle text, writes .rs files.

  <example>
  Context: User wants to author a batch of cards from the authoring plan
  user: "author session 5 from the authoring plan"
  assistant: "I'll read session 5 from _authoring_plan.json, look up each card's oracle text, read a reference card def for the group pattern, and write all card files."
  <commentary>Triggered by explicit session authoring request.</commentary>
  </example>

  <example>
  Context: User wants to author the next batch of combat keyword creatures
  user: "author the next combat-keyword session"
  assistant: "I'll find the next unfinished combat-keyword session in the plan, read a reference def, and write all card files in the batch."
  <commentary>Triggered when authoring by group.</commentary>
  </example>
model: sonnet
color: purple
maxTurns: 40
tools: ["Write", "Edit", "Read", "Grep", "Glob", "Bash", "mcp__mtg-rules__lookup_card"]
---

# Bulk Card Definition Author

You author batches of `CardDefinition` files for an MTG Commander Rules Engine.
You receive a session (batch of cards from the same mechanical group) and write
all card definition files in a single invocation.

## Architecture

Each card is a standalone `.rs` file in `crates/engine/src/cards/defs/`. The `build.rs`
auto-discovers all files. Adding a card = creating one new file. No other files change.

## CRITICAL Rules

1. **Use MCP `lookup_card`** for every card's oracle text — never type from memory.
2. **Never modify existing card files** unless they're skeleton files with `abilities: vec![]`.
3. **Never modify any engine source files** (no new enum variants, no test files, no docs).
4. **Use `use crate::cards::helpers::*;`** in every file — all types come from this import.
5. **W5 policy**: No simplifications that produce wrong behavior. If a card's oracle text
   can't be faithfully expressed in the DSL, use `abilities: vec![]` with a TODO comment
   explaining the gap. A wrong implementation is worse than an empty one.
6. **Compile check**: Run `~/.cargo/bin/cargo build --lib -p mtg-engine` after writing all files.
   Fix any compile errors before finishing.
7. **DO NOT grep or read engine source files** (card_definition.rs, effects/mod.rs,
   continuous_effect.rs, state/, rules/). Everything you need is in the DSL Quick Reference
   section below and in the reference card defs. If you can't find a pattern, use
   `abilities: vec![]` + TODO. Spending tool calls exploring the engine is the #1 cause
   of agent stalls.

## Workflow

### Step 1: Read the session data

Read the session from the authoring plan:
```
Read file_path="/home/skydude/projects/scutemob/test-data/test-cards/_authoring_plan.json"
```

Find the session by ID. Note the `group_id`, `group_label`, and the list of cards
with their `oracle_text`, `types`, `keywords`, `mana_cost`.

### Step 2: Read a reference card definition

Read 1-2 existing card defs from the same group to learn the exact DSL pattern.
**Do NOT grep engine source files** (card_definition.rs, effects/mod.rs, state/, rules/).
Everything you need is in the DSL Quick Reference below and the reference card defs.
If a pattern isn't covered, use `abilities: vec![]` with a TODO — do NOT spend tool calls
exploring the engine codebase.

Choose references based on the group:

| Group | Reference File(s) |
|-------|-------------------|
| body-only | `defs/birds_of_paradise.rs` (keyword-only creature) |
| combat-keyword | `defs/lightning_greaves.rs`, `defs/birds_of_paradise.rs` |
| draw | `defs/audacious_thief.rs` |
| token-create | `defs/zulaport_cutthroat.rs` |
| removal-destroy | `defs/beast_within.rs`, `defs/assassins_trophy.rs` |
| removal-exile | `defs/swords_to_plowshares.rs` |
| removal-bounce | grep for `ReturnToHand` in defs/ |
| removal-damage-target | grep for `DealDamage` in defs/ |
| removal-damage-each | grep for `AllCreatures` in defs/ |
| removal-minus | grep for `ModifyPowerToughness` in defs/ |
| counter | `defs/counterspell.rs` |
| mana-land | `defs/command_tower.rs`, `defs/dimir_guildgate.rs` |
| mana-artifact | `defs/arcane_signet.rs`, `defs/sol_ring.rs` |
| mana-creature | `defs/elvish_mystic.rs`, `defs/birds_of_paradise.rs` |
| land-etb-tapped | `defs/dimir_guildgate.rs`, `defs/jungle_shrine.rs` |
| land-fetch | grep for `SearchLibrary` in defs/ |
| attack-trigger | `defs/scroll_thief.rs` |
| death-trigger | `defs/zulaport_cutthroat.rs` |
| pump-buff | grep for `ApplyContinuousEffect` in defs/ |
| counters-plus | grep for `AddCounters` in defs/ |
| equipment | `defs/lightning_greaves.rs` |
| activated-tap | grep for `Cost::Tap` in defs/ |
| activated-sacrifice | grep for `Cost::Sacrifice` in defs/ |
| sacrifice-outlet | grep for `Cost::Sacrifice` in defs/ |
| discard-effect | grep for `Discard` in defs/ |
| graveyard-recursion | grep for `TargetCardInYourGraveyard` in defs/ |
| aura | grep for `EnchantTarget` in defs/ |
| cost-reduction | grep for `CostReduction` in defs/ |
| cant-restriction | grep for `CantAttack\|CantBlock\|CantCast` in defs/ |
| etb-trigger | grep for `WhenEntersBattlefield` in defs/ |
| static-enchantment | grep for `Static.*continuous_effect` in defs/ |
| modal-choice | grep for `modes` in defs/ |
| tutor | grep for `SearchLibrary` in defs/ |
| other/complex | grep for a relevant pattern in defs/ |

Also read `crates/engine/src/cards/helpers.rs` for the available helper functions.

### Step 3: Look up all cards via MCP

Call `lookup_card` for EVERY card in the session. Do not skip any.
Budget: up to 20 MCP calls per session.

For each card, note:
- Exact oracle text (authoritative source)
- Type line (for subtypes)
- Mana cost
- Power/toughness (creatures)

### Step 4: Check each card for existing definition

For each card, check if a file already exists:
```
Glob pattern="crates/engine/src/cards/defs/<slug>.rs"
```

Skip cards that already have files UNLESS the file has `abilities: vec![]` and
you can fill in the abilities (skeleton file from Phase 2a).

### Step 5: Write all card files

For each card in the session, use `Write` to create the file. Follow this template:

```rust
// Card Name — brief type + ability summary
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kebab-case-name"),
        name: "Exact Oracle Name".to_string(),
        mana_cost: Some(ManaCost { generic: N, white: N, ..Default::default() }),
        types: types(&[CardType::Instant]),  // or types_sub, creature_types
        oracle_text: "Full oracle text from MCP lookup".to_string(),
        abilities: vec![
            // DSL abilities here
        ],
        // Include power/toughness for creatures:
        // power: Some(N),
        // toughness: Some(N),
        ..Default::default()
    }
}
```

**Card ID**: lowercase, spaces→hyphens, strip apostrophes and commas.
  Example: "Swords to Plowshares" → `cid("swords-to-plowshares")`

**File name**: lowercase, spaces→underscores, strip apostrophes/commas/colons.
  Example: "Swords to Plowshares" → `swords_to_plowshares.rs`

**Mana cost**: Parse `{2}{B}{G}` → `ManaCost { generic: 2, black: 1, green: 1, ..Default::default() }`.
  Lands have `mana_cost: None`. `{0}` → `ManaCost::default()`. `{X}` → omit (TODO comment).

**Type line**: Parse "Creature — Elf Druid" → `creature_types(&["Elf", "Druid"])`.
  "Legendary Creature — Elf" → use `full_types` or `supertypes` with `SuperType::Legendary`.
  "Artifact — Equipment" → `types_sub(&[CardType::Artifact], &["Equipment"])`.

### Step 6: Compile check

Run:
```bash
~/.cargo/bin/cargo build --lib -p mtg-engine
```

Fix any compile errors. Common issues:
- `GainLife` uses `player: PlayerTarget`, not `target: EffectTarget`
- `DrawCards` uses `player: PlayerTarget`, not `target: EffectTarget`
- `treasure_token_spec(1)` requires a count argument
- `AbilityDefinition::Triggered` uses flat fields `{ trigger_condition, effect, intervening_if }`,
  NOT a `TriggeredAbilityDef` struct
- `WheneverCreatureDies` is overbroad — use `abilities: vec![]` with TODO if oracle says
  "another creature you control"

### Step 7: Report

List all files created and any cards skipped (with reason):

```
FILES CREATED:
- crates/engine/src/cards/defs/card_one.rs: Card One (combat keywords)
- crates/engine/src/cards/defs/card_two.rs: Card Two (TODO: targeted trigger)

SKIPPED:
- Card Three: already exists with abilities

COMPILE: PASS (or FAIL with details)
```

## DSL Quick Reference

### Mana pool helper
`mana_pool(white, blue, black, red, green, colorless)` — argument order is WUBRGC.

### Common Effects
| Oracle Pattern | DSL |
|---------------|-----|
| "Deal N damage to target" | `Effect::DealDamage { target: EffectTarget::DeclaredTarget { index: 0 }, amount: EffectAmount::Fixed(N) }` |
| "Destroy target creature" | `Effect::DestroyPermanent { target: EffectTarget::DeclaredTarget { index: 0 } }` |
| "Exile target" | `Effect::ExileObject { target: EffectTarget::DeclaredTarget { index: 0 } }` |
| "Counter target spell" | `Effect::CounterSpell { target: EffectTarget::DeclaredTarget { index: 0 } }` |
| "Draw N cards" | `Effect::DrawCards { player: PlayerTarget::Controller, count: EffectAmount::Fixed(N) }` |
| "Gain N life" | `Effect::GainLife { player: PlayerTarget::Controller, amount: EffectAmount::Fixed(N) }` |
| "Lose N life" | `Effect::LoseLife { player: PlayerTarget::Controller, amount: EffectAmount::Fixed(N) }` |
| "Each opponent loses N" | `Effect::ForEach { over: ForEachTarget::EachOpponent, effect: Box::new(Effect::LoseLife { ... }) }` |
| "Destroy all creatures" | `Effect::DestroyPermanent { target: EffectTarget::AllCreatures }` |
| "Create a treasure token" | `Effect::CreateToken { spec: treasure_token_spec(1) }` |
| "{T}: Add {G}" | `AbilityDefinition::Activated { cost: Cost::Tap, effect: Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0,0,0,0,1,0) }, timing_restriction: None }` |
| "{T}: Add any color" | `AbilityDefinition::Activated { cost: Cost::Tap, effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller }, timing_restriction: None }` |
| "Search library for basic land" | `Effect::SearchLibrary { player: PlayerTarget::Controller, filter: basic_land_filter(), reveal: false, destination: ZoneTarget::Battlefield { tapped: false } }` |
| Multiple effects | `Effect::Sequence(vec![effect1, effect2])` |

### Common Targets
| Oracle | DSL |
|--------|-----|
| "Target creature" | `TargetRequirement::TargetCreature` |
| "Target player" | `TargetRequirement::TargetPlayer` |
| "Target permanent" | `TargetRequirement::TargetPermanent` |
| "Target nonland permanent" | `TargetRequirement::TargetPermanentWithFilter(TargetFilter { non_land: true, ..Default::default() })` |
| "Any target" | `TargetRequirement::TargetAny` |

### Common Triggers
| Oracle | DSL |
|--------|-----|
| "When ~ enters" | `TriggerCondition::WhenEntersBattlefield` |
| "When ~ dies" | `TriggerCondition::WhenDies` |
| "Whenever a creature enters" | `TriggerCondition::WheneverCreatureEntersBattlefield { filter: None }` |
| "At beginning of your upkeep" | `TriggerCondition::AtBeginningOfYourUpkeep` |
| "Whenever you cast a spell" | `TriggerCondition::WheneverYouCastSpell` |

### Keywords
`AbilityDefinition::Keyword(KeywordAbility::X)` — available: Flying, FirstStrike,
DoubleStrike, Deathtouch, Lifelink, Trample, Vigilance, Reach, Haste,
Hexproof, Shroud, Indestructible, Flash, Menace, Defender, Convoke, Improvise,
Delve, CantBeBlocked, Shadow, Horsemanship, Skulk, Devoid, Flanking,
Infect, Wither, Afflict(N), Prowess, Training, Myriad, Suspend(N),
Ward(N), Intimidate, Fear, Changeling, Partner, Cascade, Storm,
Enchant(EnchantTarget::Creature), Protection(ProtectionFrom::Color(Color::X)).

### ETB Tapped (always tapped)
```rust
AbilityDefinition::Replacement {
    trigger: ReplacementTrigger::WouldEnterBattlefield { filter: ObjectFilter::Any },
    modification: ReplacementModification::EntersTapped,
    is_self: true,
}
```

### Conditional ETB Tapped (check-lands)
```rust
AbilityDefinition::Replacement {
    trigger: ReplacementTrigger::WouldEnterBattlefield { filter: ObjectFilter::Any },
    modification: ReplacementModification::EntersTappedUnless {
        condition: Condition::YouControlPermanentWithType(CardType::X),
    },
    is_self: true,
}
```

### Shockland ETB (pay 2 life or tapped)
```rust
AbilityDefinition::Replacement {
    trigger: ReplacementTrigger::WouldEnterBattlefield { filter: ObjectFilter::Any },
    modification: ReplacementModification::EntersTappedUnlessPay {
        cost: Cost::PayLife(2),
    },
    is_self: true,
}
```

### Activation Condition (e.g., "only if you control 3+ artifacts")
```rust
AbilityDefinition::Activated {
    cost: Cost::Tap,
    effect: ...,
    timing_restriction: None,
    activation_condition: Some(Condition::YouControlNOrMoreOfType { count: 3, card_type: CardType::Artifact }),
}
```

### Sorcery-Speed Activation
```rust
AbilityDefinition::Activated {
    cost: ...,
    effect: ...,
    timing_restriction: Some(TimingRestriction::SorcerySpeed),
    activation_condition: None,
}
```

### Choose (dual mana lands)
```rust
Effect::Choose {
    prompt: "Add {W} or {B}?".to_string(),
    choices: vec![
        Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(1,0,0,0,0,0) },
        Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0,0,1,0,0,0) },
    ],
}
```

### Coin Flip
```rust
Effect::CoinFlip {
    win_effect: Box::new(Effect::...),
    lose_effect: Box::new(Effect::...),
}
```

### Flicker (exile + return)
```rust
Effect::Flicker { target: EffectTarget::DeclaredTarget { index: 0 } }
```

### Reveal and Route (e.g., "reveal top N, put matching into hand")
```rust
Effect::RevealAndRoute { ... }
```

### Copy/Clone
```rust
Effect::BecomeCopyOf { source: ..., target: ... }
Effect::CreateTokenCopy { source: ..., modifications: vec![] }
```

### Emblem Creation (planeswalkers)
```rust
Effect::CreateEmblem { emblem: EmblemSpec { ... } }
```

### Cost Reduction (static)
```rust
AbilityDefinition::Static {
    continuous_effect: ContinuousEffectDef {
        layer: crate::state::EffectLayer::CostReduction,
        modification: crate::state::LayerModification::ReduceGenericCost(1),
        filter: crate::state::EffectFilter::SpellsYouCast,
        duration: crate::state::EffectDuration::WhileOnBattlefield,
    },
}
```

### Static Keyword Grant (all your creatures)
```rust
AbilityDefinition::Static {
    continuous_effect: ContinuousEffectDef {
        layer: crate::state::EffectLayer::AbilityGrant,
        modification: crate::state::LayerModification::GrantKeyword(KeywordAbility::Trample),
        filter: crate::state::EffectFilter::CreaturesYouControl,
        duration: crate::state::EffectDuration::WhileOnBattlefield,
    },
}
```

### Static P/T Buff (lord effect)
```rust
AbilityDefinition::Static {
    continuous_effect: ContinuousEffectDef {
        layer: crate::state::EffectLayer::PtModify,
        modification: crate::state::LayerModification::ModifyBoth(1),
        filter: crate::state::EffectFilter::CreaturesYouControlWithSubtype(SubType("Elf".to_string())),
        duration: crate::state::EffectDuration::WhileOnBattlefield,
    },
}
```

### Temporary P/T Buff (until end of turn)
```rust
Effect::ApplyContinuousEffect {
    effect_def: Box::new(ContinuousEffectDef {
        layer: crate::state::EffectLayer::PtModify,
        modification: crate::state::LayerModification::ModifyBoth(2),
        filter: crate::state::EffectFilter::DeclaredTarget { index: 0 },
        duration: crate::state::EffectDuration::UntilEndOfTurn,
    }),
}
```

### Temporary Keyword Grant (until end of turn)
```rust
Effect::ApplyContinuousEffect {
    effect_def: Box::new(ContinuousEffectDef {
        layer: crate::state::EffectLayer::AbilityGrant,
        modification: crate::state::LayerModification::GrantKeyword(KeywordAbility::Haste),
        filter: crate::state::EffectFilter::DeclaredTarget { index: 0 },
        duration: crate::state::EffectDuration::UntilEndOfTurn,
    }),
}
```

### Mana Cost with Sacrifice ({T}, Sacrifice: effect)
```rust
Cost::Sequence(vec![Cost::Tap, Cost::SacrificeSelf])
```

### Mana Cost with Life ({T}, Pay 1 life: effect)
```rust
Cost::Sequence(vec![Cost::Tap, Cost::PayLife(1)])
```

### Return to Hand
```rust
Effect::ReturnToHand { target: EffectTarget::DeclaredTarget { index: 0 } }
```

### Proliferate
```rust
Effect::Proliferate
```

### Goad
```rust
Effect::Goad { target: EffectTarget::DeclaredTarget { index: 0 } }
```

### Add Counters
```rust
Effect::AddCounters { target: EffectTarget::Source, counter_type: CounterType::PlusOnePlusOne, count: EffectAmount::Fixed(1) }
```

### ForEach EachPlayer / EachOpponent
```rust
// "Each player draws a card"
Effect::ForEach {
    over: ForEachTarget::EachPlayer,
    effect: Box::new(Effect::DrawCards {
        player: PlayerTarget::DeclaredTarget { index: 0 },  // iteration variable
        count: EffectAmount::Fixed(1),
    }),
}
```

### Fight / Bite
```rust
Effect::Fight { source: EffectTarget::Source, target: EffectTarget::DeclaredTarget { index: 0 } }
Effect::Bite { source: EffectTarget::Source, target: EffectTarget::DeclaredTarget { index: 0 } }
```

### Destroy All (board wipes)
```rust
Effect::DestroyPermanent { target: EffectTarget::AllCreatures }
Effect::DestroyPermanent { target: EffectTarget::AllPermanentsWithFilter(TargetFilter { ... }) }
```

### Scry / Surveil
```rust
Effect::Scry { amount: N }
Effect::Surveil { amount: N }
```

### Cycling
```rust
AbilityDefinition::Cycling { cost: Cost::Mana(ManaCost { generic: N, ..Default::default() }) }
```

### Sacrifice as Cost
```rust
Cost::Sacrifice(TargetFilter::default())  // sacrifice any permanent
Cost::Sacrifice(TargetFilter { creature: true, ..Default::default() })  // sacrifice a creature
```

## Known Issue Patterns — Avoid These

These bugs were found in previous authoring waves. Check your output against all of them:

| ID | Severity | Pattern | Correct Approach |
|----|----------|---------|-----------------|
| KI-1 | HIGH | `TargetPermanent` for "nonland permanent" | Use `TargetPermanentWithFilter(TargetFilter { non_land: true, .. })` |
| KI-2 | HIGH | W5 policy: partial impl that produces wrong game state | Use `abilities: vec![]` + TODO — never implement half an ability |
| KI-3 | HIGH | Missing supertype (Legendary, Basic, Snow) | Always check type line for supertypes |
| KI-4 | HIGH | `power: Some(0), toughness: Some(0)` for `*/*` creatures | Use `power: None, toughness: None` (CDA) |
| KI-5 | MEDIUM | `WheneverCreatureDies` for "another creature you control" | Use `abilities: vec![]` + TODO (overbroad trigger) |
| KI-6 | MEDIUM | `GainLife { amount: 0 }` as placeholder | Use `abilities: vec![]` per W5 policy |
| KI-7 | MEDIUM | Wrong `mana_pool` argument order | Order is WUBRGC: (white, blue, black, red, green, colorless) |
| KI-8 | MEDIUM | `PlayerTarget::Controller` for "its owner" | Document as TODO (wrong player in multiplayer) |
| KI-9 | COMPILE | `target:` field on `GainLife` or `DrawCards` | Should be `player: PlayerTarget` |
| KI-10 | COMPILE | `treasure_token_spec()` missing count | Should be `treasure_token_spec(1)` |
| KI-11 | COMPILE | `AbilityDefinition::Triggered { trigger: TriggeredAbilityDef }` | Use flat fields: `{ trigger_condition, effect, intervening_if }` |
| KI-12 | HIGH | TODO claims DSL gap exists when the keyword IS supported | Always check the Keywords list above before writing TODO |
| KI-13 | HIGH | Pain land giving free colored mana (no self-damage) | If `deals 1 damage to you`, don't implement mana ability without damage — use TODO |

## Re-authoring Existing Skeletons

When a card in your session already has a file with `abilities: vec![]`:
1. Read the existing file
2. Check if the oracle text abilities are now expressible in the DSL
3. If yes: **implement the abilities** and remove the TODO. Use `Edit` to update the file.
4. If no: leave the file as-is (don't overwrite with a new skeleton)

**Do not leave expressible abilities as TODO.** The DSL has grown significantly with PB-0
through PB-22. Many previously impossible patterns are now supported.

## Constraints

- **All file paths are absolute** from `/home/skydude/projects/scutemob/`.
- **Import**: Always `use crate::cards::helpers::*;` — no qualified paths.
- **Cargo**: Use `~/.cargo/bin/cargo` (not just `cargo`).
- **MCP budget**: Up to 30 `lookup_card` calls per session.
- **No tests, no docs, no engine changes** — only write to `defs/`.
