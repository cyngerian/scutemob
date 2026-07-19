# Primitive WIP — PB-OS9 (IN PROGRESS)

<!-- last_updated: 2026-07-19 -->

**Batch**: PB-OS9 — Lieutenant / "you control your commander" condition (OOS-EF3b-1)
**Task**: `scutemob-139`
**Branch**: `feat/pb-os9-lieutenant-condition-you-control-your-commander-oos-e`
**Queue plan**: `memory/primitives/oos-retriage-plan-2026-07-18.md` §3 (PB-OS9), §5 notes
**Seed**: OOS-EF3b-1 (filed by PB-EF3b, `ef-batch-plan-2026-07-17.md`)

## Phase
- [x] plan  → `memory/primitives/pb-plan-OS9.md`
- [x] implement  → see progress below; DEVIATION from plan, see note
- [x] review  → `memory/primitives/pb-review-OS9.md` (0 HIGH / 0 MEDIUM, ship-ready)
- [ ] fix  → none required

## Implement progress
- [x] Change 1: `Condition::YouControlYourCommander` unit variant added
      (`crates/card-types/src/cards/card_definition.rs`, after `YouAttackedWithNOrMore`)
- [x] Change 2: real evaluator arm in `check_condition`
      (`crates/engine/src/effects/mod.rs`, before `SacrificeFired` arm; uses
      `state.expect_player(ctx.controller)` not a bare `.get` — SR-4/bare-lookup-ratchet)
- [x] Change 3: `check_static_condition` fallback comment (no new arm)
- [x] Change 4: hash arm discriminant 51 (`crates/engine/src/state/hash.rs`)
- [x] Change 5: wire re-pin — PROTOCOL 23→24, HASH 60→61, both `tests/core/*`
      gates green; bulk sentinel update across ~40 test files done
- [x] Card def fixes:
      - `skyhunter_strike_force.rs` → **Complete** (continuous-grant condition;
        no dependency on the AtBeginningOfCombat gap, see below)
      - `loyal_apprentice.rs` → **partial** (DEVIATION from plan — see note)
      - `siege_gang_lieutenant.rs` → **partial** (DEVIATION from plan — see note)
      - `legion_lieutenant.rs` → confirmed OUT (name-only), untouched
- [x] Unit tests (`crates/engine/tests/primitives/pb_os9_lieutenant_commander_control.rs`,
      15 tests, all green; `mod` line registered in `tests/primitives/main.rs` per SR-9a)
- [x] All gates green: `cargo test --all`, `cargo clippy --workspace -- -D warnings`,
      `cargo build --workspace`, `cargo fmt --check`, `tools/check-defs-fmt.sh`

## DEVIATION FROM PLAN — genuine engine gap found by execution (SR-34/36)

The plan assumed `TriggerCondition::AtBeginningOfCombat` card-def triggers already
fire in real gameplay (citing `legion_warboss`/`goblin_rabblemaster`/etc. as
precedent). **Verified false by execution**: `crates/engine/src/rules/turn_actions.rs`
hardcodes a per-step card-def trigger sweep for AtBeginningOfYourUpkeep /
AtBeginningOfFirstMainPhase / AtBeginningOfPostcombatMain / AtBeginningOfYourEndStep,
but `begin_combat()` (`Step::BeginningOfCombat`) only queues EMBLEM triggers
(`collect_emblem_triggers_for_event`) — never card-defined
`AbilityDefinition::Triggered { trigger_condition: AtBeginningOfCombat, .. }`
abilities. A scratch probe test (placed + removed) confirmed zero pending triggers /
zero stack objects after a real `Command::PassPriority` transition into
`BeginningOfCombat` for a battlefield object with this trigger condition.

This is PRE-EXISTING (also silently affects `legion_warboss.rs`,
`goblin_rabblemaster.rs`, `mirage_phalanx.rs`, `helm_of_the_host.rs` — none touched
here, out of scope) and blocks `loyal_apprentice`/`siege_gang_lieutenant`'s
Lieutenant trigger from ever firing in real gameplay. Per the no-wrong-game-state /
no-gated-stub-as-Complete guardrail, both stayed `Completeness::partial` with a
"STILL BLOCKED" note (their Lieutenant DSL is CR-correct and uses
`Condition::YouControlYourCommander` correctly — it's the trigger QUEUEING that's
broken, not this primitive). Tests for those two cards isolate the intervening-if-
at-resolution mechanism (CR 603.4) by queueing the exact `PendingTrigger` the
missing sweep would produce, directly — proving the primitive itself is correct and
ready to fire once that sweep is added.

**Recommend filing a new seed**: "card-def `AtBeginningOfCombat` sweep in
`begin_combat()` (turn_actions.rs)" — would flip `loyal_apprentice`,
`siege_gang_lieutenant`, `legion_warboss`, `goblin_rabblemaster`, `mirage_phalanx`,
`helm_of_the_host` closer to/fully Complete. NOT fixed here (out of PB-OS9's Engine
Changes scope; flagged per the runner's STOP-AND-REPORT instruction rather than
silently expanded).

**Discounted ship**: 1 clean flip (`skyhunter_strike_force`), not the plan's
predicted 3 — `loyal_apprentice` and `siege_gang_lieutenant` stay partial pending
the new engine-gap seed above.

## WIRE STOP-AND-FLAG (AC 5083) — RESOLVED AT PLAN TIME
**A PROTOCOL + HASH bump IS machine-forced.** `Condition` is in BOTH the SR-8 PROTOCOL
closure (reachable via `Effect::Conditional`; PB-OS6 precedent forced 20→21) and the
GameState hash closure (`HashInto for Condition`). A new unit variant
`YouControlYourCommander` moves both shape digests.
- **PROTOCOL_VERSION 23 → 24** (gate: `tests/core/protocol_schema.rs`), fingerprint re-pinned.
- **HASH_SCHEMA_VERSION 60 → 61** (gate: `tests/core/hash_schema.rs`), decl+stream re-pinned.
Single batched bump taken (both axes, one commit). Independently verified:
PROTOCOL_VERSION=23 @ protocol.rs:220, HASH_SCHEMA_VERSION=60 @ hash.rs:543.
No-bump was NOT an option — do not silently absorb (PB-OS7 precedent).

## Scope
`Condition::YouControlYourCommander` (new variant) checked against the effect
controller's `commander_ids` + battlefield presence **+ CONTROL** (CR 903 — a
stolen commander you do not control does NOT count; a commander you stole back
DOES). Two consumption shapes both needed by existing defs:
1. **Intervening-if** on a triggered ability — `loyal_apprentice`, `siege_gang_lieutenant`.
2. **Continuous-grant condition** (layer re-eval on commander enter/leave/control-change)
   — `skyhunter_strike_force` (grants Melee to other creatures you control).

## Candidates (verify oracle via MCP at plan time)
- `skyhunter_strike_force` (partial→Complete) — continuous grant.
- `loyal_apprentice` (TODO Lieutenant clause) — intervening-if trigger.
- `siege_gang_lieutenant` (TODO Lieutenant clause) — intervening-if trigger.
- `legion_lieutenant` — NOT a Lieutenant-ability card (Vampire lord; "Lieutenant"
  only in name). OUT of scope; confirm.

## Wire impact (SR-8 — MUST verify at plan time, stop-and-flag either way)
`Condition` is in the SR-8 PROTOCOL closure. A new `Condition` variant is a
machine-forced PROTOCOL bump (precedent PB-OS7). Take the single bump if forced.
Determine at plan time.

## Guardrails
- Decoys MUST cover: commander-on-battlefield → grant/trigger active; commander
  in command zone → inactive; commander dies/exiled → drops (SBA batch semantics);
  STOLEN commander decoy (opponent controls YOUR commander → Lieutenant OFF).
- Do NOT widen into OOS-EF9-1's WhileSourceOnBattlefield reversion half or
  OOS-OS7-2's 611.2c edge.
- Probe by execution (SR-34/36); enumerate roster from `all_cards()`.
