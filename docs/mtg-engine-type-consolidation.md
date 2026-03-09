# Type Consolidation Plan: Ability System Refactoring

> **Purpose**: Reduce type proliferation across the ability system before M10 networking.
> Five refactoring clusters consolidate ~360 enum variants and ~78 one-off struct fields
> into generalized patterns. This is the sole active work until complete.
>
> **Status**: COMPLETE (all 8 sessions done, 2026-03-09)
> **Created**: 2026-03-09
> **Blocks**: All other workstreams until complete

---

## Motivation

After 93 ability implementations across 16 batches + 3 mini-milestones, the engine's core
types have accumulated significant structural debt:

- **StackObjectKind**: 62 variants, ~25 of which are one-off trigger wrappers
- **PendingTriggerKind**: 45+ variants mirroring SOK trigger variants
- **CastSpell**: 32 fields, ~20 of which are ability-specific `Option<T>` or `Vec<T>`
- **GameObject**: 48+ ability-specific fields (booleans, counters, relationship IDs)
- **AbilityDefinition**: 64 variants, ~8 of which are graveyard-cast variants sharing identical structure

The `/implement-ability` pipeline's planner agent studies existing abilities to plan new ones.
If these patterns remain scattered, future implementations will copy the scattered pattern.
Consolidating now ensures the remaining ~20 unvalidated abilities and all future card authoring
follow clean, generalized types.

**M10 urgency**: CastSpell is the Command variant that gets serialized over the wire. 32 fields
(most `Option::None` for any given spell) is wasteful and fragile for network serialization.

---

## Refactoring Clusters

### RC-1: CastSpell Additional Cost Consolidation (HIGH priority)

**Current state**: 20 ability-specific fields on `Command::CastSpell`:

```
// Sacrifice-as-cost fields (all Option<ObjectId> or Vec<ObjectId>)
bargain_sacrifice: Option<ObjectId>
emerge_sacrifice: Option<ObjectId>
casualty_sacrifice: Option<ObjectId>
devour_sacrifices: Vec<ObjectId>

// Discard-as-cost fields
retrace_discard_land: Option<ObjectId>
jump_start_discard: Option<ObjectId>

// Payment tracking
assist_player: Option<PlayerId>
assist_amount: u32
replicate_count: u32
escalate_modes: u32
squad_count: u32
collect_evidence_cards: Vec<ObjectId>
splice_cards: Vec<ObjectId>

// Boolean cost flags
entwine_paid: bool
offspring_paid: bool
fuse: bool

// Player choice
gift_opponent: Option<PlayerId>

// Mutate
mutate_target: Option<ObjectId>
mutate_on_top: bool
```

**Target state**: Replace with a `Vec<AdditionalCost>` enum:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdditionalCost {
    // Sacrifice patterns (Bargain, Emerge, Casualty, Devour)
    Sacrifice(Vec<ObjectId>),
    // Discard patterns (Retrace, Jump-Start)
    Discard(Vec<ObjectId>),
    // Exile from zone (Collect Evidence, Escape)
    ExileFromZone { cards: Vec<ObjectId> },
    // Player assistance (Assist)
    Assist { player: PlayerId, amount: u32 },
    // Replication (Replicate, Squad)
    Replicate { count: u32 },
    // Mode payment (Escalate)
    EscalateModes { count: u32 },
    // Splice onto
    Splice { cards: Vec<ObjectId> },
    // Entwine (all modes)
    Entwine,
    // Fuse (both halves)
    Fuse,
    // Offspring token
    Offspring,
    // Gift choice
    Gift { opponent: PlayerId },
    // Mutate target
    Mutate { target: ObjectId, on_top: bool },
}
```

**CastSpell after refactor** (retained fields):
```rust
Command::CastSpell {
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
    // Cost reduction (kept — these modify mana, not additional costs)
    convoke_creatures: Vec<ObjectId>,
    improvise_artifacts: Vec<ObjectId>,
    delve_cards: Vec<ObjectId>,
    // Core spell properties (kept)
    kicker_times: u32,
    alt_cost: Option<AltCostKind>,
    prototype: bool,
    x_value: u32,
    modes_chosen: Vec<usize>,
    face_down_kind: Option<FaceDownKind>,
    // NEW: all additional costs in one vec
    additional_costs: Vec<AdditionalCost>,
}
```

Reduces CastSpell from 32 fields to 14 fields.

**Files to modify**:
- `crates/engine/src/rules/command.rs` — CastSpell definition
- `crates/engine/src/state/stack.rs` — StackObject mirrored fields
- `crates/engine/src/state/game_object.rs` — GameObject mirrored fields
- `crates/engine/src/rules/casting.rs` — cost payment logic
- `crates/engine/src/rules/resolution.rs` — resolution reads these fields
- `crates/engine/src/rules/abilities.rs` — ability activation
- `crates/engine/src/testing/replay_harness.rs` — harness action translation
- `crates/engine/src/state/hash.rs` — hash the new type
- `crates/engine/src/state/builder.rs` — builder defaults
- `crates/engine/src/cards/helpers.rs` — export AdditionalCost
- `crates/engine/tests/*.rs` — all test files constructing CastSpell (~95 files)
- `crates/simulator/src/legal_actions.rs` — LegalActionProvider
- `tools/replay-viewer/src/view_model.rs` — display
- `tools/tui/src/play/panels/stack_view.rs` — display

**Migration strategy**: Define `AdditionalCost` enum first. Add `additional_costs: Vec<AdditionalCost>`
to CastSpell. Migrate one cost category at a time (sacrifice fields first, then discard, etc.).
Remove old fields only after all reads/writes are migrated. Run `cargo test --all` after each
category migration.

**escape_exile_cards note**: Currently on CastSpell but conceptually an alt-cost detail (Escape).
Move to `AdditionalCost::ExileFromZone` since it's a mandatory additional cost component of
Escape casting, not a property of AltCostKind itself.

---

### RC-2: StackObjectKind Trigger Consolidation (HIGH priority)

**Current state**: 25+ one-off trigger variants that all follow the pattern
`XxxTrigger { source_object: ObjectId, ...captured_data }`.

**Trigger variant groups**:

| Group | Current Variants | Shared Pattern |
|-------|-----------------|----------------|
| Counter-removal upkeep | VanishingCounterTrigger, VanishingSacrificeTrigger, FadingTrigger, EchoTrigger, CumulativeUpkeepTrigger, ImpendingCounterTrigger | Upkeep → check counter/cost → sacrifice if failed |
| Combat modifier | FlankingTrigger, RampageTrigger, ProvokeTrigger, MeleeTrigger, PoisonousTrigger, EnlistTrigger | Combat event → modify creature/player |
| ETB data-capture | ExploitTrigger, BackupTrigger, ChampionETBTrigger, SoulbondTrigger, GraftTrigger, SquadTrigger, OffspringTrigger, GiftETBTrigger | Permanent ETB → captured data drives effect |
| Spell copy | CasualtyTrigger, ReplicateTrigger, GravestormTrigger, StormTrigger, CascadeTrigger | Spell cast → create N copies |
| End-of-turn sacrifice | DashReturnTrigger, BlitzSacrificeTrigger, EncoreSacrificeTrigger, UnearthTrigger | EOT → sacrifice/exile/return |
| Death/exile | ModularTrigger, HauntExileTrigger, HauntedCreatureDiesTrigger, ChampionLTBTrigger, RecoverTrigger | Dies/LTB → effect |

**Target state**: Introduce `TriggerData` enum to carry captured data, consolidate groups:

```rust
/// Captured data for triggered abilities on the stack.
/// Replaces 25+ one-off StackObjectKind variants.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TriggerData {
    // No extra data needed
    Simple,
    // Counter-removal upkeep triggers (Vanishing, Fading, Impending)
    CounterRemoval { permanent: ObjectId },
    // Echo/CumulativeUpkeep (cost-based upkeep)
    UpkeepCost { permanent: ObjectId, cost: UpkeepCostKind },
    // Combat modifier triggers
    CombatModifier { modifier: CombatTriggerKind },
    // ETB with target/count
    ETBWithTarget { target: Option<ObjectId>, count: u32, grants: Vec<KeywordAbility> },
    // ETB token creation (Squad, Offspring)
    ETBTokenCreation { source_card_id: Option<CardId>, count: u32 },
    // Gift ETB
    GiftETB { source_card_id: Option<CardId>, gift_opponent: PlayerId },
    // Spell copy triggers (Storm, Cascade, Casualty, Replicate, Gravestorm)
    SpellCopy { original_stack_id: ObjectId, copy_count: u32 },
    // Cascade-style (exile-until-hit)
    CascadeExile { spell_mana_value: u32 },
    // End-of-turn zone change
    DelayedZoneChange { destination: ZoneType },
    // Dies/LTB data capture
    DeathCapture { captured_data: DeathCaptureKind },
    // Recover from graveyard
    Recover { card: ObjectId, cost: ManaCost },
    // Cipher combat damage
    CipherDamage { creature: ObjectId, encoded_card_id: CardId, encoded_object_id: ObjectId },
    // Champion exile/return pair
    ChampionExile { filter: ChampionFilter },
    ChampionReturn { exiled_card: ObjectId },
    // Soulbond pairing
    Soulbond { pair_target: ObjectId },
    // Ravenous draw check
    RavenousDraw { permanent: ObjectId, x_value: u32 },
    // Bloodrush pump
    Bloodrush { target: ObjectId, power: i32, toughness: i32, keyword: Option<KeywordAbility> },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UpkeepCostKind {
    Echo(ManaCost),
    CumulativeUpkeep(CumulativeUpkeepCost),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CombatTriggerKind {
    Flanking { blocker: ObjectId },
    Rampage { n: u32 },
    Provoke { target: ObjectId },
    Melee,
    Poisonous { target_player: PlayerId, n: u32 },
    Enlist { enlisted: ObjectId },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeathCaptureKind {
    Modular { counter_count: u32 },
    HauntExile { card_id: Option<CardId> },
    HauntedCreatureDies { card_id: Option<CardId> },
}
```

**New StackObjectKind** (collapsed):
```rust
pub enum StackObjectKind {
    // Unchanged (these carry unique resolution logic, not just data)
    Spell { source_object: ObjectId },
    ActivatedAbility { source_object: ObjectId, ability_index: usize, embedded_effect: Option<Box<Effect>> },
    TriggeredAbility { source_object: ObjectId, ability_index: usize },
    MutatingCreatureSpell { source_object: ObjectId, target: ObjectId },

    // Consolidated trigger (replaces ~25 variants)
    KeywordTrigger { source_object: ObjectId, keyword: KeywordAbility, data: TriggerData },

    // Kept separate (unique activation/resolution paths, not just triggers)
    NinjutsuAbility { ... },
    UnearthAbility { ... },
    EmbalmAbility { ... },
    EternalizeAbility { ... },
    EncoreAbility { ... },
    ForecastAbility { ... },
    ScavengeAbility { ... },
    BloodrushAbility { ... },  // OR move to KeywordTrigger with TriggerData::Bloodrush
    SaddleAbility { ... },
    CraftAbility { ... },
    TransformTrigger { ... },

    // Kept separate (unique stack behavior)
    MadnessTrigger { ... },
    MiracleTrigger { ... },
    SuspendCounterTrigger { ... },
    SuspendCastTrigger { ... },
}
```

**Estimated reduction**: 62 → ~20 SOK variants. The `KeywordTrigger` variant absorbs ~25 trigger
variants. Activated ability variants (Ninjutsu, Unearth, etc.) stay separate because they have
distinct resolution paths.

**PendingTriggerKind parallel change**: Consolidate the matching 25+ PTK variants into a
parallel `KeywordTrigger { keyword, data }` variant. Resolution in `resolution.rs` dispatches
on `keyword` + `data` instead of on the PTK variant.

**Files to modify**:
- `crates/engine/src/state/stack.rs` — SOK definition + all impl blocks
- `crates/engine/src/state/stubs.rs` — PendingTriggerKind definition
- `crates/engine/src/rules/resolution.rs` — trigger resolution dispatch (~500 lines)
- `crates/engine/src/rules/abilities.rs` — trigger creation
- `crates/engine/src/rules/combat.rs` — combat trigger creation
- `crates/engine/src/rules/casting.rs` — spell copy triggers
- `crates/engine/src/effects/mod.rs` — effect-triggered triggers
- `tools/replay-viewer/src/view_model.rs` — display match
- `tools/tui/src/play/panels/stack_view.rs` — display match

**Migration strategy**: Add `KeywordTrigger` variant first. Migrate one group at a time
(counter-removal first, then combat, etc.). Each group: update creation sites → update
resolution match → remove old variant → `cargo test --all`. Keep activated-ability SOK
variants untouched.

---

### RC-3: Graveyard-Cast AbilityDefinition Consolidation (MEDIUM priority)

**Current state**: 8 AbilityDefinition variants with identical structure `{ cost: ManaCost }`:

```
Flashback { cost: ManaCost }
Embalm { cost: ManaCost }
Eternalize { cost: ManaCost }
Encore { cost: ManaCost }
Unearth { cost: ManaCost }
Dash { cost: ManaCost }      // hand, not graveyard — but same structure
Blitz { cost: ManaCost }     // hand, not graveyard — but same structure
Plot { cost: ManaCost }      // exile, not graveyard — but same structure
```

Plus variants with extra fields:
```
Escape { cost: ManaCost, exile_count: u32 }
Aftermath { name: String, cost: ManaCost, card_type: CardType, effect: Effect, targets: Vec<TargetRequirement> }
```

**Target state**: Group by casting-zone source:

```rust
pub enum AbilityDefinition {
    // ... existing non-alt-cost variants unchanged ...

    /// Alternative casting cost ability (Flashback, Escape, Unearth, Embalm, etc.)
    /// The AltCostKind discriminant determines resolution behavior.
    AltCastAbility {
        kind: AltCostKind,
        cost: ManaCost,
        /// Extra data for abilities that need it
        details: Option<AltCastDetails>,
    },

    // ... Aftermath stays separate (it carries an entire spell half) ...
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AltCastDetails {
    Escape { exile_count: u32 },
    Eternalize { name_override: Option<String> },
    Prototype { power: i32, toughness: i32 },
}
```

**Estimated reduction**: 64 → ~56 AbilityDefinition variants (8 collapsed into 1).

**Files to modify**:
- `crates/engine/src/cards/card_definition.rs` — AbilityDefinition enum
- `crates/engine/src/cards/helpers.rs` — export AltCastDetails
- `crates/engine/src/rules/casting.rs` — alt-cost detection
- `crates/engine/src/rules/abilities.rs` — graveyard activation detection
- `crates/simulator/src/legal_actions.rs` — legal action generation
- `crates/engine/src/cards/defs/*.rs` — all card definitions using these abilities (~30 files)

**Migration strategy**: Add `AltCastAbility` variant. Migrate one ability at a time (Flashback
first as simplest). Update card defs in bulk per ability. Run tests after each.

---

### RC-4: GameObject Designation Bitfield (LOW priority, LOW effort)

**Current state**: 8 boolean designation fields on GameObject:

```
is_renowned: bool
is_suspected: bool
is_saddled: bool
echo_pending: bool
is_bestowed: bool
is_foretold: bool
is_suspended: bool
is_reconfigured: bool
```

**Target state**: Pack into a bitfield:

```rust
bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct Designations: u16 {
        const RENOWNED       = 0b0000_0000_0001;
        const SUSPECTED      = 0b0000_0000_0010;
        const SADDLED        = 0b0000_0000_0100;
        const ECHO_PENDING   = 0b0000_0000_1000;
        const BESTOWED       = 0b0000_0001_0000;
        const FORETOLD       = 0b0000_0010_0000;
        const SUSPENDED      = 0b0000_0100_0000;
        const RECONFIGURED   = 0b0000_1000_0000;
        // Room for 8 more without widening to u32
    }
}
```

Replace `obj.is_renowned` with `obj.designations.contains(Designations::RENOWNED)` etc.
Setter: `obj.designations.insert(Designations::RENOWNED)`.

**Files to modify**:
- `crates/engine/src/state/game_object.rs` — struct definition
- `crates/engine/src/state/hash.rs` — hash the u16 instead of 8 bools
- `crates/engine/src/state/builder.rs` — default
- All files that read/write these fields (grep for each field name)
- `Cargo.toml` — add `bitflags` dependency

**Migration strategy**: Add `Designations` type + `designations` field with default. Migrate one
field at a time: add accessor method, replace all reads/writes, remove old field. Trivial but
touches many files.

---

### RC-5: Counter-Removal Upkeep Trigger Unification (LOW priority, LOW effort)

This is subsumed by RC-2 — the counter-removal triggers become
`KeywordTrigger { keyword: Vanishing/Fading/etc., data: TriggerData::CounterRemoval { permanent } }`.
No separate work item needed.

---

## Session Plan

### Session 1: Foundation Types + RC-4 (bitfield)

**Goal**: Define new types, implement the simplest refactor (RC-4) end-to-end as proof-of-concept.

1. Add `bitflags` to engine Cargo.toml
2. Define `Designations` bitfield in `game_object.rs`
3. Add `designations: Designations` field to GameObject (alongside old fields temporarily)
4. Migrate all 8 boolean fields one-by-one: add accessor, update all read/write sites, remove old field
5. Update `hash.rs`, `builder.rs`, token creation in `effects/mod.rs`
6. `cargo test --all && cargo clippy -- -D warnings`
7. **Review**: Run `milestone-reviewer` agent to audit all changes. Fix any HIGH/MEDIUM findings before proceeding. Add LOWs to `docs/mtg-engine-milestone-reviews.md`.

**Agent**: `session-runner` (manual — no session plan file, work directly from this doc)
**Estimated effort**: 1-2 hours
**Risk**: Low — purely mechanical replacement

---

### Session 2: RC-1 CastSpell Consolidation — Define Type + Sacrifice Fields

**Goal**: Define `AdditionalCost` enum, migrate sacrifice-related fields.

1. Define `AdditionalCost` enum in `crates/engine/src/state/types.rs`
2. Add `additional_costs: Vec<AdditionalCost>` to CastSpell (keep old fields temporarily)
3. Export from `helpers.rs`
4. Migrate `bargain_sacrifice` → `AdditionalCost::Sacrifice`
5. Migrate `emerge_sacrifice` → `AdditionalCost::Sacrifice`
6. Migrate `casualty_sacrifice` → `AdditionalCost::Sacrifice`
7. Migrate `devour_sacrifices` → `AdditionalCost::Sacrifice`
8. Update StackObject mirrored fields
9. Update resolution.rs, casting.rs reads
10. Update replay_harness.rs action translation
11. Update all test CastSpell construction sites
12. Remove old fields
13. `cargo test --all && cargo clippy -- -D warnings`
14. **Review**: Run `milestone-reviewer` agent to audit all changes. Fix any HIGH/MEDIUM findings before proceeding. Add LOWs to `docs/mtg-engine-milestone-reviews.md`.

**Agent**: `session-runner` (manual from this doc)
**Estimated effort**: 3-4 hours (bulk of time is test file updates)
**Risk**: Medium — touching ~95 test files, but changes are mechanical

---

### Session 3: RC-1 CastSpell Consolidation — Remaining Fields

**Goal**: Migrate all remaining CastSpell additional cost fields.

1. Migrate `retrace_discard_land`, `jump_start_discard` → `AdditionalCost::Discard`
2. Migrate `escape_exile_cards`, `collect_evidence_cards` → `AdditionalCost::ExileFromZone`
3. Migrate `assist_player`, `assist_amount` → `AdditionalCost::Assist`
4. Migrate `replicate_count`, `squad_count` → `AdditionalCost::Replicate`
5. Migrate `escalate_modes` → `AdditionalCost::EscalateModes`
6. Migrate `splice_cards` → `AdditionalCost::Splice`
7. Migrate `entwine_paid` → `AdditionalCost::Entwine`
8. Migrate `fuse` → `AdditionalCost::Fuse`
9. Migrate `offspring_paid` → `AdditionalCost::Offspring`
10. Migrate `gift_opponent` → `AdditionalCost::Gift`
11. Migrate `mutate_target`, `mutate_on_top` → `AdditionalCost::Mutate`
12. Remove all old fields, remove from StackObject/GameObject mirrors
13. `cargo test --all && cargo clippy -- -D warnings`
14. `cargo build --workspace` (verify replay-viewer + TUI compile)
15. **Review**: Run `milestone-reviewer` agent to audit all changes. Fix any HIGH/MEDIUM findings before proceeding. Add LOWs to `docs/mtg-engine-milestone-reviews.md`.

**Agent**: `session-runner` (manual from this doc)
**Estimated effort**: 4-5 hours
**Risk**: Medium — same mechanical pattern as Session 2

---

### Session 4: RC-2 SOK Trigger Consolidation — Define Types + Counter-Removal Group

**Goal**: Define `TriggerData`/`CombatTriggerKind`/etc., migrate counter-removal triggers.

1. Define `TriggerData`, `UpkeepCostKind`, `CombatTriggerKind`, `DeathCaptureKind` in `stack.rs`
2. Add `KeywordTrigger { source_object, keyword, data }` variant to StackObjectKind
3. Add parallel `KeywordTrigger { keyword, data }` variant to PendingTriggerKind
4. Migrate `VanishingCounterTrigger` → `KeywordTrigger { keyword: Vanishing, data: CounterRemoval }`
5. Migrate `VanishingSacrificeTrigger` → same pattern
6. Migrate `FadingTrigger` → same pattern
7. Migrate `EchoTrigger` → `KeywordTrigger { keyword: Echo, data: UpkeepCost { Echo(cost) } }`
8. Migrate `CumulativeUpkeepTrigger` → same pattern
9. Migrate `ImpendingCounterTrigger` → same pattern
10. Update resolution.rs dispatch for migrated triggers
11. Update abilities.rs/turn_actions.rs trigger creation
12. Remove old variants
13. `cargo test --all && cargo clippy -- -D warnings`
14. **Review**: Run `milestone-reviewer` agent to audit all changes. Fix any HIGH/MEDIUM findings before proceeding. Add LOWs to `docs/mtg-engine-milestone-reviews.md`.

**Agent**: `session-runner` (manual from this doc)
**Estimated effort**: 3-4 hours
**Risk**: Medium — resolution.rs dispatch is complex but each trigger has isolated logic

---

### Session 5: RC-2 SOK Trigger Consolidation — Combat + ETB + Spell-Copy Groups

**Goal**: Migrate remaining trigger groups.

1. Migrate combat triggers (Flanking, Rampage, Provoke, Melee, Poisonous, Enlist) → `KeywordTrigger { data: CombatModifier }`
2. Migrate ETB triggers (Exploit, Backup, Squad, Offspring, Gift, Graft, Champion, Soulbond, Ravenous) → `KeywordTrigger { data: ETBWithTarget/ETBTokenCreation/etc. }`
3. Migrate spell-copy triggers (Casualty, Replicate, Gravestorm, Storm, Cascade) → `KeywordTrigger { data: SpellCopy/CascadeExile }`
4. Migrate EOT triggers (DashReturn, BlitzSacrifice, EncoreSacrifice, Unearth) → `KeywordTrigger { data: DelayedZoneChange }`
5. Migrate death triggers (Modular, Haunt variants, ChampionLTB, Recover) → `KeywordTrigger { data: DeathCapture/Recover/etc. }`
6. Migrate remaining (Cipher, Ingest, Hideaway, PartnerWith, Renown) → appropriate TriggerData
7. Update all resolution.rs dispatch, creation sites, display matches
8. Remove all old variants
9. Update view_model.rs and stack_view.rs
10. `cargo test --all && cargo build --workspace`
11. **Review**: Run `milestone-reviewer` agent to audit all changes. Fix any HIGH/MEDIUM findings before proceeding. Add LOWs to `docs/mtg-engine-milestone-reviews.md`.

**Agent**: `session-runner` (manual from this doc)
**Estimated effort**: 5-6 hours (largest session — many variants)
**Risk**: High — touching resolution.rs heavily. Run tests after each group, not just at end.

---

### Session 6: RC-3 AbilityDefinition Consolidation

**Goal**: Consolidate graveyard/alt-cast AbilityDefinition variants.

1. Define `AltCastDetails` enum in `card_definition.rs`
2. Add `AltCastAbility { kind, cost, details }` variant to AbilityDefinition
3. Migrate Flashback → `AltCastAbility { kind: Flashback, cost, details: None }`
4. Migrate Embalm, Eternalize, Encore, Unearth, Dash, Blitz, Plot → same pattern
5. Migrate Escape → `AltCastAbility { kind: Escape, cost, details: Some(Escape { exile_count }) }`
6. Migrate Prototype → `AltCastAbility { kind: Prototype, cost, details: Some(Prototype { power, toughness }) }`
7. Update casting.rs alt-cost detection
8. Update abilities.rs graveyard activation
9. Update legal_actions.rs
10. Update ~30 card definition files
11. Remove old variants
12. `cargo test --all && cargo build --workspace`
13. **Review**: Run `milestone-reviewer` agent to audit all changes. Fix any HIGH/MEDIUM findings before proceeding. Add LOWs to `docs/mtg-engine-milestone-reviews.md`.

**Agent**: `session-runner` (manual from this doc)
**Estimated effort**: 3-4 hours
**Risk**: Medium — card def updates are bulk but mechanical

---

### Session 7: Memory & Documentation Updates

**Goal**: Update all memory files, gotchas, conventions, and docs so the agent pipeline
and future sessions use the new patterns.

1. **`memory/conventions.md`** — Add section: "Ability Type Patterns"
   - Document `AdditionalCost` usage for new abilities
   - Document `KeywordTrigger` + `TriggerData` for new triggers
   - Document `AltCastAbility` for new alt-cost abilities
   - Document `Designations` bitfield for new boolean flags

2. **`memory/gotchas-infra.md`** — Update:
   - Remove all references to old field names (bargain_sacrifice, emerge_sacrifice, etc.)
   - Update "Discriminant chain" notes to reference KeywordTrigger pattern
   - Update EOC Flag Pattern section to use Designations
   - Update CastSpell construction guidance
   - Add new gotcha: "AdditionalCost extraction helpers" (how to query costs)
   - Add new gotcha: "KeywordTrigger dispatch pattern in resolution.rs"

3. **`memory/MEMORY.md`** — Update:
   - Clean batch handoff notes that reference old field names
   - Update "Behavioral Gotchas" section
   - Add "Type Consolidation" section noting what changed and when
   - Update discriminant chain documentation

4. **`docs/mtg-engine-ability-coverage.md`** — Run `/audit-abilities` to refresh

5. **`docs/ability-batch-plan.md`** — Add "Post-Batch Consolidation" section noting the refactor

6. **`CLAUDE.md`** — Update:
   - "What Exists" section: note consolidated types
   - Update `Current State` with new variant counts
   - Add reference to this plan doc in Primary Documents table

7. **`crates/engine/src/cards/helpers.rs`** — Verify all new types exported:
   - `AdditionalCost`
   - `AltCastDetails`
   - `TriggerData`, `CombatTriggerKind`, `UpkeepCostKind`, `DeathCaptureKind`
   - `Designations`

8. **Agent prompt awareness** — Verify by reading `.claude/agents/*.md`:
   - `ability-impl-planner` studies existing abilities (will see new patterns automatically)
   - `ability-impl-runner` follows the plan (plans will reference new types)
   - `card-definition-author` reads helpers.rs (will see new exports)
   - No agent prompt changes needed if helpers.rs + conventions.md are updated

9. `cargo test --all && cargo clippy -- -D warnings && cargo build --workspace`
10. **Review**: Run `milestone-reviewer` agent to audit all changes. Fix any HIGH/MEDIUM findings before proceeding. Add LOWs to `docs/mtg-engine-milestone-reviews.md`.

**Agent**: Manual (this is documentation work, not code)
**Estimated effort**: 1-2 hours
**Risk**: Low — but critical for downstream correctness

---

### Session 8: Validation + Strategic Review Refresh

**Goal**: Verify everything works end-to-end, refresh the strategic review.

1. Run full test suite: `cargo test --all`
2. Run clippy: `cargo clippy -- -D warnings`
3. Run workspace build: `cargo build --workspace`
4. Run 3-5 game scripts via SCRIPT_FILTER to verify replay harness
5. Spot-check 3 card definitions compile and behave correctly
6. Update `docs/mtg-engine-strategic-review.md` with current metrics:
   - New variant counts (SOK, PTK, AbilDef, CastSpell fields)
   - Updated test count
   - Updated card/script counts
   - Note the consolidation as a completed pre-M10 action item
7. **Review**: Run `milestone-reviewer` agent for a final comprehensive review of all type consolidation changes (RC-1 through RC-4). Fix any HIGH/MEDIUM findings. Add LOWs to `docs/mtg-engine-milestone-reviews.md`.
8. Commit: `chore: type consolidation complete — RC-1 through RC-4`

**Agent**: `milestone-reviewer` (required — final gate before commit)
**Estimated effort**: 1-2 hours
**Risk**: Low

---

## Progress Tracker

| Session | Cluster | Status | Date | Notes |
|---------|---------|--------|------|-------|
| 1 | RC-4: Designation bitfield | **COMPLETE** | 2026-03-09 | 8 bools → 1 u16 bitfield; all tests pass, clippy clean, workspace builds |
| 2 | RC-1: CastSpell (sacrifice fields) | **COMPLETE** | 2026-03-09 | 4 sacrifice fields → AdditionalCost::Sacrifice; 14 AdditionalCost variants defined; 1 MEDIUM fixed (devour heuristic); 7 LOW |
| 3 | RC-1: CastSpell (remaining fields) | **COMPLETE** | 2026-03-09 | 16 remaining fields migrated; CastSpell 32→13 fields; StackObject -9 mirrored fields; CastWithMutate command removed; 6 LOW |
| 4 | RC-2: SOK triggers (counter-removal) | **COMPLETE** | 2026-03-09 | 6 counter-removal triggers → KeywordTrigger; TriggerData/UpkeepCostKind defined; PTK lost Copy derive; 3 LOW |
| 5 | RC-2: SOK triggers (combat+ETB+copy+EOT+death) | **COMPLETE** | 2026-03-09 | ~30 triggers → KeywordTrigger; 34 TriggerData variants; 1 MEDIUM fixed (catch-all→unreachable); 4 LOW |
| 6 | RC-3: AbilityDefinition consolidation | **COMPLETE** | 2026-03-09 | 10 variants → AltCastAbility { kind, cost, details }; AbilDef 64→55; AltCastDetails enum (Escape, Prototype); 5 new AltCostKind variants; ~30 files updated; 0 issues |
| 7 | Memory & documentation updates | **COMPLETE** | 2026-03-09 | conventions.md, gotchas-infra.md, MEMORY.md, CLAUDE.md all updated; helpers.rs exports verified; 1934 tests, clippy clean, workspace builds |
| 8 | Validation + strategic review refresh | **COMPLETE** | 2026-03-09 | 5 scripts validated (126/129/132/133/136); strategic review updated; gate review: 0 HIGH, 0 MEDIUM, 3 LOW (MR-TC-23/24/25); 1934 tests, clippy clean, workspace builds |

**Metrics before refactoring** (2026-03-09):
- CastSpell fields: 32
- StackObjectKind variants: 62
- PendingTriggerKind variants: 45
- AbilityDefinition variants: 64
- GameObject designation booleans: 8
- Tests passing: ~1900

**Metrics after refactoring** (2026-03-09):
- CastSpell fields: 13 (was 32)
- StackObjectKind variants: ~20 (was 62)
- PendingTriggerKind variants: ~20 (was 45)
- AbilityDefinition variants: 55 (was 64)
- GameObject designation booleans: 0 (bitfield, u16)
- Tests passing: 1934

---

## Completion Criteria

- [x] All `cargo test --all` pass (1934)
- [x] `cargo clippy -- -D warnings` clean
- [x] `cargo build --workspace` compiles (engine + replay-viewer + TUI)
- [x] CastSpell reduced from 32 to 13 fields
- [x] StackObjectKind reduced from 62 to ~20 variants
- [x] PendingTriggerKind reduced proportionally (~20)
- [x] AbilityDefinition reduced from 64 to 55 variants
- [x] GameObject boolean designations packed into bitfield
- [x] All memory files updated (conventions.md, gotchas-infra.md, MEMORY.md)
- [x] helpers.rs exports all new types
- [x] Strategic review refreshed with current metrics
- [ ] Single commit with all changes

## Risk Mitigation

- **Run tests after every sub-step**, not just at session end
- **Keep old and new fields parallel** during migration — remove old only after all reads migrated
- **One group at a time** — never migrate two groups simultaneously
- **`cargo build --workspace`** after every session to catch replay-viewer/TUI breakage
- **If any session takes >6 hours**, stop and reassess scope — consider deferring remaining groups

## Estimated Total Effort

| Session | Effort | Cumulative |
|---------|--------|------------|
| 1: Foundation + RC-4 | 1-2h | 1-2h |
| 2: RC-1 sacrifice fields | 3-4h | 4-6h |
| 3: RC-1 remaining fields | 4-5h | 8-11h |
| 4: RC-2 counter-removal triggers | 3-4h | 11-15h |
| 5: RC-2 remaining triggers | 5-6h | 16-21h |
| 6: RC-3 AbilityDefinition | 3-4h | 19-25h |
| 7: Memory & docs | 1-2h | 20-27h |
| 8: Validation + review | 1-2h | 21-29h |
| **Total** | **21-29h** | ~8 sessions |
