# Primitive WIP: PB-36 -- Evasion/protection extensions

batch: PB-36
title: Evasion/protection extensions
cards_affected: ~21
started: 2026-03-29
phase: closed
plan_file: memory/primitives/pb-plan-36.md

## Gap Groups
- G-31: Evasion/protection statics (~21 cards) — "can't be blocked except by N+ creatures" (Menace variant), player protection, extend CantBeBlocked filter

## Deferred from Prior PBs
- none specific to evasion/protection

## Step Checklist
- [x] 1. Engine changes (new types/variants/dispatch) — BlockingExceptionFilter enum + CantBlock(160)/CantBeBlockedExceptBy(161) in KeywordAbility; GrantPlayerProtection effect(73); combat.rs enforcement; hash.rs; view_model.rs; mod.rs+helpers.rs re-exports
- [x] 2. Card definition fixes — bloodghast/carrion_feeder/phoenix_chick/skrelv/vishgraz/skrevls_hive/white_suns_twilight (CantBlock); signal_pest/gingerbrute (CantBeBlockedExceptBy); emrakul/greensleeves/sword_of_body_and_mind/cryptic_coat/untimely_malfunction (protection wiring); teferis_protection/the_one_ring (GrantPlayerProtection)
- [x] 3. New card definitions (if any) — N/A, no new card defs needed
- [x] 4. Unit tests — 9 tests in crates/engine/tests/evasion_protection.rs; all pass
- [x] 5. Workspace build verification — cargo build --workspace clean; clippy clean; fmt clean; 2428 tests pass

## Review
findings: 9 (HIGH: 0, MEDIUM: 3, LOW: 6)
verdict: needs-fix → fixed
review_file: memory/primitives/pb-review-36.md

## Fix Phase (2026-03-29)
- [x] MEDIUM 1: hash.rs GrantPlayerProtection — added `(qualities.len() as u64).hash_into(hasher)` before quality loop
- [x] MEDIUM 2: hash.rs BlockingExceptionFilter::HasAnyKeyword — added `(kws.len() as u64).hash_into(hasher)` before kw loop
- [x] MEDIUM 3: combat.rs Provoke section — added CantBeBlockedExceptBy impossibility check after CantBeBlocked check
- 2428 tests pass; 0 clippy warnings
