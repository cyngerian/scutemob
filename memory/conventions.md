# Conventions — Last verified: M9.5 + Type Consolidation Complete (S1-S8)

## Rust Style

- **Edition**: 2021
- **Formatting**: `rustfmt` default settings. Run `cargo fmt` before every commit.
- **Linting**: `cargo clippy --workspace --all-targets -- -D warnings` (the CI bar; the
  bare `cargo clippy` skips every test target and misses unused-import errors there). No
  warnings allowed in CI.
- **Error handling**: `thiserror` for library errors, `anyhow` in binaries/tools only.
  Engine crate uses typed errors — never `unwrap()` or `expect()` in engine logic. Tests
  may use `unwrap()`.
- **Naming**: Types `PascalCase`, functions/methods `snake_case`, constants
  `SCREAMING_SNAKE_CASE`, modules `snake_case`.

## Comprehensive Rules Citation Format

Every rules implementation MUST cite the CR section it implements:

```rust
/// Implements CR 704.5f: "If a creature has toughness 0 or less, it's put into
/// its owner's graveyard."
fn check_zero_toughness(state: &GameState) -> Vec<GameEvent> { ... }
```

For tests, cite the rule AND the source of the test case:

```rust
#[test]
/// CR 704.5f — creature with 0 toughness dies as SBA
/// Source: CR example under 704.5f
fn test_704_5f_zero_toughness_creature_dies() { ... }

#[test]
/// CR 613.10 — Humility + Opalescence interaction
/// Source: CR example under 613.10, confirmed by Forge engine
fn test_613_10_humility_opalescence() { ... }
```

## Testing Conventions

- **Test location**: `crates/engine/tests/`, not inline `#[cfg(test)]` modules. Black-box
  testing against the public API only.
- **GameStateBuilder**: Always use the builder. Never manually construct `GameState` structs
  — the builder ensures invariants.
- **One assertion focus per test**: single behavior per test; multiple related assertions are
  fine, but the test name should describe the specific behavior.
- **Test naming**: `test_<system>_<scenario>_<expected_behavior>`
  - Good: `test_sba_creature_zero_toughness_goes_to_graveyard`
  - Good: `test_priority_all_four_players_pass_stack_resolves`
  - Bad: `test_combat` (too vague), `test_1` (meaningless)
- **Golden test format**: JSON files in `test-data/golden-games/`. Schema in architecture
  doc §6.4.
- **Property tests**: Use `proptest` crate. Define invariants in `tests/properties/`.
- **Full-dispatch tests for new `LayerModification` variants**: every new variant added to `LayerModification` MUST ship with at least one test that exercises the full dispatch path — effect application, layer resolution, dispatch-site reads, and a game-state mutation verifying the behavior through `calculate_characteristics` after full layer resolution. Not a direct unit test of the substitution function. Established after PB-S (discriminant 23 was unreachable at runtime, surfaced by a retroactive L2 test that caught 2 HIGH bugs neither plan nor review noticed). Reinforced by PB-X (C1 HIGH — Obelisk's observability-window bug was invisible until `test_obelisk_of_urd_chosen_type_pump` exercised the post-ETB pre-priority characteristics read).

## Commit Conventions

- **Format**: `M<number>: <short description>` (e.g., `M1: implement GameState struct`)
- **PR scope**: One logical change per PR.
- **Tests required**: Every PR touching engine logic must include or update tests.
- **Benchmark check**: If PR touches state cloning, layer calculation, or SBA checks, run
  benchmarks and note any regression.

## Type Consolidation Patterns (2026-03-09, ongoing)

Active refactoring plan: `docs/mtg-engine-type-consolidation.md`. Read before modifying
core types (GameObject, CastSpell, StackObjectKind, AbilityDefinition).

### Designations Bitfield (RC-4, COMPLETE)

Boolean designation flags on `GameObject` use the `Designations` bitflags type, NOT individual
`bool` fields. The 8 migrated flags are: RENOWNED, SUSPECTED, SADDLED, ECHO_PENDING, BESTOWED,
FORETOLD, SUSPENDED, RECONFIGURED.

```rust
// Reading:
if obj.designations.contains(Designations::RENOWNED) { ... }

// Setting:
obj.designations.insert(Designations::RENOWNED);

// Clearing:
obj.designations.remove(Designations::SADDLED);

// Default (all false):
designations: Designations::default(),
```

When adding a new boolean designation to `GameObject`, add a new flag to `Designations` (u16,
room for 8 more). Do NOT add a new `bool` field.

### AdditionalCost Enum (RC-1, COMPLETE — Sessions 2-3)

CastSpell additional cost fields (sacrifice, discard, splice, etc.) consolidated into
`additional_costs: Vec<AdditionalCost>`. New abilities that add casting costs should add an
`AdditionalCost` variant, NOT a new field on CastSpell.

```rust
// Adding a sacrifice cost (Bargain, Emerge, Casualty, Devour):
additional_costs: vec![AdditionalCost::Sacrifice(vec![obj_id])]

// Adding a discard cost (Retrace, Jump-Start):
additional_costs: vec![AdditionalCost::Discard(vec![card_id])]

// Exile from zone (Escape, Collect Evidence):
additional_costs: vec![AdditionalCost::ExileFromZone { cards: vec![id1, id2] }]

// Query: check if a specific cost was paid
cast.additional_costs.iter().any(|c| matches!(c, AdditionalCost::Sacrifice(_)))
```

### KeywordTrigger SOK (RC-2, COMPLETE — Sessions 4-5)

One-off StackObjectKind trigger variants consolidated into
`KeywordTrigger { source_object, keyword, data: TriggerData }`. New keyword triggers should
add a `TriggerData` variant, NOT a new SOK variant.

```rust
// Creating a keyword trigger:
StackObjectKind::KeywordTrigger {
    source_object: obj_id,
    keyword: KeywordAbility::Vanishing(3),
    data: TriggerData::CounterRemoval { permanent: obj_id },
}

// Matching in resolution.rs:
StackObjectKind::KeywordTrigger { keyword, data, .. } => {
    match (keyword, data) {
        (KeywordAbility::Vanishing(_), TriggerData::CounterRemoval { permanent }) => { ... }
        ...
    }
}
```

New triggers should add a `TriggerData` variant, NOT a new SOK variant.

### AltCastAbility (RC-3, COMPLETE — Session 6)

Alt-cost AbilityDefinition variants consolidated into `AltCastAbility { kind: AltCostKind, cost: ManaCost, details: Option<AltCastDetails> }`. New graveyard/alt-cost abilities MUST use this variant, NOT add a new AbilityDefinition variant.

```rust
// Simple alt-cost (Flashback, Embalm, Eternalize, Encore, Unearth, Dash, Blitz, Plot):
AbilityDefinition::AltCastAbility { kind: AltCostKind::Flashback, cost: mana_cost, details: None }

// Escape (with exile count):
AbilityDefinition::AltCastAbility {
    kind: AltCostKind::Escape,
    cost: mana_cost,
    details: Some(AltCastDetails::Escape { exile_count: 3 }),
}

// Prototype (with alt P/T):
AbilityDefinition::AltCastAbility {
    kind: AltCostKind::Prototype,
    cost: mana_cost,
    details: Some(AltCastDetails::Prototype { power: 3, toughness: 3 }),
}
```

Cost extraction: use `get_alt_cast_cost(abilities, AltCostKind::X)` pattern — scan abilities for matching `AltCastAbility { kind, cost, .. }`.

## Dependencies Policy

- **Engine crate**: `im`, `serde`, `thiserror`, `bitflags`. No async runtime, no IO, no network, no UI.
- **Network crate**: `tokio`, `tokio-tungstenite` or `axum`, `serde`, `rmp-serde`.
- **Card-db crate**: `rusqlite`, `serde`.
- **Tauri app**: `tauri`, `serde`, frontend deps.

Engine crate must NEVER depend on network, card-db, or tauri-app crates. Information flows
inward only: app depends on network, network depends on engine. Never the reverse.

## Review & Fix Discipline (PB pipeline)

### Test-validity MEDIUMs are fix-phase HIGHs

Any review finding of the form *"test exists but doesn't validate what its name promises"*
is a **fix-phase HIGH**, regardless of the severity the reviewer initially tagged. The
PB-Q4 retro established that silent-skip tests are the exact failure mode we are trying
to extinguish; deferring them as LOWs perpetuates the pattern. PB-N F3/F4 reinforced this
when test 6 (LKI wedge) and test 9 (combat_damage_filter regression) both passed against
both pre-fix and post-fix engines.

**Rule**: if the test title says "pre-death LKI" and the setup can't discriminate pre- vs
post-death evaluation, that is a test-validity bug with the same urgency as a
wrong-game-state bug. Fix-phase must rewrite the test or escalate to the coordinator;
never log it as a LOW.

### Hash sentinel convention

Hash schema version lives as a `pub const` in `crates/engine/src/state/hash.rs`,
referenced at:
1. The literal hash arm where the sentinel is written into the hash stream
2. The parity test assertion (`assert_eq!(HASH_SCHEMA_VERSION, <N>)`, not `assert_ne!(hash, [0u8; 32])`)
3. Re-exported from `crates/engine/src/lib.rs` for test access

Non-zero assertions on the sentinel are too weak — they pass against rollbacks and
forks. The strict equality form catches both.

**Hash bump rule**: bump on every change to a serialized type's field shape or variant
shape. Default action: bump. Stop-and-flag is only required if the change is to a
derived/computed field that does not affect serialization. The cost of bumping is
near-zero (one constant edit + one test parity assertion); the cost of *not* bumping when
an old replay file deserializes against new state is real and silent. Document the bump
in the implement commit message and in the hash module comment.

### Implement-phase default-to-defer (new standing rule, PB-N)

During fix-phase work, if a finding requires new engine surface beyond the declared PB
scope, **stop and flag**, do not silently extend. "I'll just add one more variant" is the
anti-pattern. The worker's job is to fix within-scope; a primitive extension is a
micro-PB and needs its own plan/review cycle.

Exception: trivial no-op extensions (e.g. re-exporting an existing constant, adding a
one-field backfill default) are allowed if they unblock multiple existing sites and do
not introduce new dispatch logic.

### Aspirationally-wrong code comments are correctness hazards

If a fix-phase investigation reveals that an existing source comment describes *intended*
behavior rather than *actual* behavior, the comment is a lie that will mislead the next
reader. Either fix the behavior (if in scope) or fix the comment to describe actual
behavior + point at the tracking LOW (if out of scope). **Never leave the aspirational
version standing.**

Originating incident: PB-N close phase found `crates/engine/src/rules/abilities.rs:4191-4193`
claiming *"Layer-resolved characteristics preserve pre-death state because
move_object_to_zone retains Characteristics on the graveyard object"* — the comment was
aspirationally correct (that's what CR 603.10a requires) but the code path called
`calculate_characteristics` instead, which re-runs layer filters against the graveyard
object and drops battlefield-gated filters. The PB-N close commit replaced the comment
with a `TODO(BASELINE-LKI-01)` pointing at the tracking LOW.

### Pair-or-demote: a source gate that stands in for a behaviour ships with its probe

Adopted 2026-09-05 (course-correction addendum A3, accepted as written; CC-17, `scutemob-254`).

A source-text gate proves a line is SPELLED a certain way. It cannot prove the line does
anything. The batch records already say so — `OOS-DX52-2`: *"a row that reddens only a source
gate is telling you the behaviour has no probe."* This makes that sentence a rule:

1. **A new source gate that stands in for a behaviour** ("site X calls helper Y", "no second
   predicate exists", "arm Z consults field W") **ships with a behavioural probe that reddens
   under the same revert**, or with a **one-line reason in the gate's own doc** why no probe can
   be built **plus a seed ID** filed for it. The change-class table (row 4) already limits a new
   gate to one executed defeat; this adds the pairing.
2. **No retroactive sweep.** An existing unpaired source gate gets its probe **at the moment a
   batch re-keys it after a defeat** — that is the occasion, and the probe is written THEN
   instead of hardening the regex. Nothing schedules a walk over the 486 tests in 53 files that
   read engine source; they are touched when they are touched.
3. **A gate that has a paired probe is a backstop.** It needs no bypass matrix when re-keyed;
   the probe is the verdict. Bypass work belongs only to gates that still stand alone.

**Exempt** (they measure a property of the source itself, which is what they are for):
exhaustiveness rosters over `all_cards()` (SR-36), the keyword registry (SR-5), the seal gate
(SR-3), the declaration and stream fingerprints (SR-8), and any ratchet whose subject is a
COUNT. Those are not proxies for a behaviour.

**Why:** the records show source gates defeated by spelling alone — a `use` alias (PB-DX36,
PB-DX49), a commented-out call (PB-DX56, `OOS-DX32-6`), an argument swap that compiles
(PB-DX56), field order (PB-DX48 `r2`), a multi-line borrow (PB-DX51 `r1d`), a `/* */` block
(PB-DX8) — and revert rows where ONLY a source gate reddened (PB-DX52 R6, PB-DX54 R2/R3,
PB-DX42b R7, PB-DX49 `r7`). Each defeat cost a re-key and a re-executed bypass; the effort spent
hardening a gate against spelling is effort not spent on the probe that would make it
unnecessary.

**Not retired by this rule (addendum A5, hard constraint):** the `HASH_SCHEMA_HISTORY` and
PROTOCOL history rows, both `FROZEN_HISTORY_PREFIX_DIGEST` pins, the declaration and stream
fingerprint gates, `[profile.fuzz]` with the HARD-equals-zero ratchet, and the SR-3 seal gate.

## Change-class acceptance table

Recorded verbatim from `docs/course-correction-2026-09.md` §3.1 item 6 (owner-approved
2026-09-05; CC-4, `scutemob-240`). **Scale the acceptance ritual to the change class.** A brief
names the class; the worker does that class's "Required" column and nothing in "Not required"
unless a finding forces it — and then says so in the notes file.

   | Change class | Required | Not required |
   |---|---|---|
   | Engine behaviour (`crates/engine/src`, `crates/card-types/src`) | suite, clippy, fmt, revert-proven probe per fix, wire prediction before code | bench A/B unless a hot-path file is touched (`layers.rs`, `sba.rs`, `priority.rs`, `combat.rs`) |
   | Card defs only | suite, `check-defs-fmt.sh`, regenerate authoring status, batch review | revert matrix, wire prediction, bench |
   | Tests / docs / tooling only | suite, clippy | everything else |
   | New source gate added | one executed defeat of the gate, recorded in the test's own doc | bypass matrix over every other gate in the batch |

Notes for applying it:

- **Hot-path files** (the bench A/B trigger in row 1): `crates/engine/src/rules/layers.rs`,
  `crates/engine/src/rules/sba.rs`, `crates/engine/src/rules/priority.rs`,
  `crates/engine/src/rules/combat.rs`. Touching any of them means a matched-set A/B against the
  merge base in an isolated worktree with the same-code band measured FIRST; nothing else does.
- "suite" is `cargo test --workspace --no-fail-fast` to a file with the count delta itemised by
  test NAME (byte-exact set difference, regex not end-anchored); "clippy" is the CI bar above;
  "fmt" is `cargo fmt --check` **plus** `tools/check-defs-fmt.sh` (SR-35).
- A batch can be in more than one class (an engine fix that also adds a source gate does rows 1
  and 4). The classes add; they do not pick the cheapest.
- Row 4 pairs with the pair-or-demote rule above ("Pair-or-demote: a source gate that stands
  in for a behaviour ships with its probe"): probe under the same revert, or a one-line reason
  plus seed ID; a paired gate needs no bypass matrix.
