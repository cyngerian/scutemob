# Primitive WIP: PB-A — Play from Top of Library

batch: PB-A
title: Play from top of library — continuous cast permission system
cards_affected: 6+
started: 2026-04-07
phase: closed
plan_file: memory/primitives/pb-plan-A.md

## Deferred from Prior PBs
- Case of the Locked Hothouse solved ability deferred to PB-A (from PB-K)
- Spelunking play-from-top note (PB-A territory, from PB-K)
- Courser of Kruphix play-from-top static (from card authoring)

## Known Cards
Cards with play-from-top TODOs in existing defs:
- Courser of Kruphix (play lands from top)
- Elven Chorus (cast creatures from top)
- Thundermane Dragon (cast creatures P4+ from top)
- Case of the Locked Hothouse (solved: play lands + cast creatures/enchantments from top)

Cards that need new defs (not yet authored):
- Future Sight (play cards from top)
- Bolas's Citadel (play nonland from top, pay life)
- Mystic Forge (cast artifact/colorless from top)
- Others TBD by planner

## Step Checklist
- [x] 1. Engine changes (new types/variants/dispatch)
  - PlayFromTopFilter enum + PlayFromTopPermission struct (state/stubs.rs)
  - play_from_top_permissions field on GameState (state/mod.rs, state/builder.rs)
  - AbilityDefinition::StaticPlayFromTop variant discriminant 73 (cards/card_definition.rs)
  - AltCostKind::PayLifeForManaValue variant (state/types.rs)
  - HashInto for PlayFromTopFilter, PlayFromTopPermission + AltCostKind::PayLifeForManaValue + StaticPlayFromTop (state/hash.rs)
  - StaticPlayFromTop registration in register_static_continuous_effects (rules/replacement.rs)
  - play_from_top_permissions cleanup in reset_turn_state (rules/turn_actions.rs)
  - casting_from_library_top detection + has_play_from_top_permission check + PayLifeForManaValue cost + on_cast_effect bonus (rules/casting.rs)
  - has_play_from_top_land_permission + zone check in handle_play_land (rules/lands.rs)
  - PlayFromTopFilter in helpers.rs exports (cards/helpers.rs)
  - PlayFromTopFilter + PlayFromTopPermission in lib.rs exports (src/lib.rs)
- [x] 2. Card definition fixes
  - Courser of Kruphix: StaticPlayFromTop { LandsOnly, reveal_top: true }
  - Elven Chorus: StaticPlayFromTop { CreaturesOnly, look_at_top: true }
  - Thundermane Dragon: StaticPlayFromTop { CreaturesWithMinPower(4), on_cast_effect haste grant }
  - Case of the Locked Hothouse: StaticPlayFromTop { CreaturesAndEnchantmentsAndLands, condition: SourceIsSolved }
- [x] 3. New card definitions
  - future_sight.rs: All filter, reveal_top: true
  - bolass_citadel.rs: All filter, look_at_top: true, pay_life_instead: true
  - mystic_forge.rs: ArtifactsAndColorless filter, look_at_top: true
  - oracle_of_mul_daya.rs: AdditionalLandPlays + LandsOnly reveal_top: true
  - vizier_of_the_menagerie.rs: CreaturesOnly look_at_top: true
  - radha_heart_of_keld.rs: conditional FirstStrike + LandsOnly look_at_top: true
- [x] 4. Unit tests
  - 16 tests in crates/engine/tests/play_from_top.rs
  - All 16 passing
- [x] 5. Workspace build verification
  - cargo test --all: all pass
  - cargo clippy -- -D warnings: clean
  - cargo build --workspace: clean
  - cargo fmt --check: clean

## Review
findings: 6 (HIGH: 2, MEDIUM: 2, LOW: 2)
verdict: needs-fix → FIXED
review_file: memory/primitives/pb-review-A.md

## Fix Phase Results
- HIGH-1: FIXED — removed continuous effect approach; added `cast_from_top_with_bonus: bool` to StackObject; resolution.rs now inserts KeywordAbility::Haste directly at resolution time using new battlefield ObjectId (CR 400.7 fix). Pattern mirrors was_dashed/was_blitzed.
- HIGH-2: FIXED — `find_play_from_top_on_cast_effect` now inlines the filter check for the specific permission being evaluated instead of delegating to `has_play_from_top_permission()` (which checked ANY permission). Future Sight + Thundermane Dragon coexistence is now correct.
- MEDIUM-3: FIXED — Added CR 119.4 life check before PayLifeForManaValue deduction at casting.rs line 3439. Returns InvalidCommand if life_total < mana value.
- MEDIUM-4: FIXED — (1) Added X=0 enforcement: when cast_with_pay_life && x_value > 0, returns InvalidCommand. (2) Added `test_play_from_top_bolas_citadel_x_is_zero` test. (3) Added `test_play_from_top_haste_grant` test verifying haste is on battlefield permanent (not just stack).
- LOW-5: documented, not fixed (mandatory life payment bypass acceptable)
- LOW-6: documented, not fixed (simulator legal_actions gap acceptable)

All 18 play_from_top tests pass. Workspace build clean. Clippy clean. fmt clean.

## Expected Remaining TODOs in Card Defs
- Elven Chorus: GrantActivatedAbility (mana ability grant) — separate gap
- Vizier of the Menagerie: mana restriction relaxation — separate gap
- Bolas's Citadel: sacrifice 10 cards activated ability — separate gap
- Mystic Forge: exile top activated ability — separate gap
- Radha, Heart of Keld: +X/+X activated ability (no dynamic ModifyBothDynamic variant) — deferred
