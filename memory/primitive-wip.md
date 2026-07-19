# Primitive WIP — PB-OS5 (OOS-EF4-1) · phase: close-out

<!-- last_updated: 2026-07-19 -->

**Task**: scutemob-135 · branch `feat/pb-os5-dynamic-relative-count-effectamount-oos-ef4-1-count-m`
**Class**: capability
**Pipeline**: /implement-primitive — planner → runner → reviewer
**Plan file**: `memory/primitives/pb-plan-OS5.md` (planner writes)
**Review file**: `memory/primitives/pb-review-OS5.md` (reviewer writes)

## Phase checklist
- [x] plan   — `primitive-impl-planner` → `pb-plan-OS5.md` (DONE — ONE new variant `OtherAttackersSharingCreatureType`; piledriver/muxus/rabblemaster reuse existing `AttackingCreatureCount`/`PermanentCount`; ×2 via `Sum(x,x)`; single PROTOCOL 19→20 + HASH 56→57)
- [x] implement — `primitive-impl-runner` (DONE 2026-07-19 — engine variant (card_definition.rs discriminant 24) + `resolve_amount` executor (effects/mod.rs) + explicit `resolve_cda_amount => 0` arm (layers.rs) + hash discriminant 24 (state/hash.rs) + PROTOCOL_VERSION 19→20 (fingerprint `5243cffc75...`) + HASH_SCHEMA_VERSION 56→57 (decl `02fb46a2f9...` / stream `31bfd0ed5d...`) + all 36 sentinel files bumped + shared_animosity→Complete + goblin_piledriver NEW→Complete + goblin_rabblemaster pump clause implemented (stays partial, note corrected) + muxus_goblin_grandee NEW attack-half (stays partial) + 11 new tests in `pb_os5_relative_attacker_count.rs` (all 3 mandatory decoys verified non-vacuous by temporary revert) — all 4 gates green)
- [x] review  — `primitive-impl-reviewer` → `pb-review-OS5.md` (CLEAN BILL — 0 HIGH, 0 MEDIUM, 2 LOW informational/no-fix)
- [x] fix     — none required (clean bill)
- [ ] gates + /review + close-out

## Seed
OOS-EF4-1 (filed by PB-EF4, `scutemob-105`) — `ef-batch-plan-2026-07-17.md` §8.
Queue entry: `oos-retriage-plan-2026-07-18.md` §3 PB-OS5.

## Fix shape (from plan)
New `EffectAmount` variant that at RESOLUTION counts battlefield objects matching a filter which
can reference the triggering/source creature's own LAYER-RESOLVED characteristics — e.g.
`OtherAttackersSharingCreatureType { relative_to }` or a more general `CountMatchingRelativeTo`.
Resolution-time count keyed on layer-resolved subtypes (changeling/type-changers must count
correctly); NO continuous-effect storage. Closest existing neighbor: `PermanentCount { filter,
controller }` (card_definition.rs:2632) — gap is (a) filter referencing a *relative* object's
layer-resolved subtypes ("shares a creature type with `relative_to`"), (b) exclude-self
(exclude `relative_to`), (c) `is_attacking` predicate + the two-arm lockstep contract
(`resolve_amount` in effects/mod.rs AND `resolve_cda_amount` in rules/layers.rs).

## Wire expectation
**NEW `EffectAmount` variant → single PROTOCOL 19→20 bump (+HASH 56→57 if the sentinel forces it).**
`EffectAmount` is inside the SR-8 fingerprint closure. Justify + update sentinels/history rows.
Also: new discriminant in `state/hash.rs`.

## Candidates (3) — ALL oracle-verified via MCP 2026-07-19
1. **shared_animosity** (`{2}{R}` Enchantment) — "Whenever a creature you control attacks, it gets
   +1/+0 UEOT **for each other attacking creature that shares a creature type with it**." Subject =
   `EffectFilter::TriggeringCreature` (PB-EF4 shipped the subject half). Count = OTHER attacking
   creatures sharing ≥1 creature type with the triggering creature. Scope = **all** attacking
   creatures regardless of controller (ruling 2008-04-01: teammate's attackers are included in the
   calc; ability counts creatures not types). Layer-resolved subtypes (a changeling/all-types
   attacker shares with everyone). Exclude-self via "other". `inert` → **Complete** candidate.
2. **goblin_piledriver** (`{1}{R}` Goblin Warrior) — "Whenever this creature attacks, it gets
   **+2/+0** UEOT **for each other attacking Goblin**." Subject = `EffectFilter::Source` (self-attack).
   Count = other attacking Goblins (FIXED subtype `Goblin`, any controller). **×2 multiplier** per
   goblin (buff = 2·count/+0). Exclude-self (PB-EF1 exclude-self precedent). Ruling: count taken on
   resolution. `known`/partial → **Complete** candidate. NOTE: also has protection-from-blue (verify
   already modeled).
3. **muxus_goblin_grandee** (`{4}{R}{R}` Goblin Noble) — attack half: "Whenever Muxus attacks, it
   gets +1/+1 UEOT **for each other Goblin you control**" (count = other Goblins YOU CONTROL, NOT
   just attacking; you-control scoped; exclude-self). **BUT** ETB half ("reveal top six, put all
   Goblin creature cards MV≤5 onto battlefield, rest on bottom in random order") is a DISTINCT
   blocker (reveal/put-from-library primitive — OOS-EF10/OS8 family). Muxus **stays `partial`** —
   name the ETB blocker honestly; do NOT flip Complete.

Discounted ship ~2 of 3 (shared_animosity + goblin_piledriver).

## Mandatory tests (from task brief)
- **Layer-resolution decoy**: a type-changed attacker counts (e.g. an attacker made a Goblin / made
  all-types); a base-typed non-attacker does NOT. Proves layer-resolved subtype read, not base.
- **Exclude-self decoy**: piledriver does not count itself (the ×2 buff excludes the source Goblin).
- **4-player decoy**: verify per-card scope exactly — shared_animosity/piledriver count attackers of
  ANY controller; muxus counts only YOUR Goblins. Do NOT assume "your attackers only".
- Probe by execution (SR-34/36): each flipped card registers + produces correct game state.

## Close-out
Close OOS-EF4-1: banner+strike PB-OS5 entry in `oos-retriage-plan-2026-07-18.md` §3; seed banner in
`ef-batch-plan-2026-07-17.md` §8; update `workstream-state.md`; reset this file to IDLE.
