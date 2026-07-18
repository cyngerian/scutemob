# Primitive WIP: PB-EF5 — card-invokable self-transform + CardType::Battle (EF-W-MISS-6)

batch: PB-EF5
title: Add Effect::TransformSelf so a triggered/activated/conditional ability can flip its own source DFC through the existing Transform/DFC machinery (CR 701.27/701.28/712). Highest single-PB card yield (11 body-only DFCs). Battle (Invasion of Ikoria) and Sephiroth "Super Nova" SPLIT OUT with justification + filed seeds.
task: scutemob-106
branch: feat/pb-ef5-card-invokable-self-transform-cardtypebattle-ef-w-mis
started: 2026-07-18
phase: done
plan_file: memory/primitives/pb-plan-EF5.md

## Outcome (done 2026-07-18)
- Effect::TransformSelf shipped (engine 29ba3b3e + card defs e3479962 + /review fix pending commit).
- New Complete: docent_of_perfection, bloodline_keeper. Demote: delver_of_secrets (integrity).
  Partial: thaumatic_compass (Spires remove-from-combat blocker, OOS-EF5-4g — /review corrected a
  fabricated ability), growing_rites_of_itlimoc (ETB blocker).
- Battle + Sephiroth SPLIT OUT → OOS-EF5-1/2; return-transformed → OOS-EF5-3; flip-conditions → OOS-EF5-4.
- Gates green: build --workspace, test --all (3396), clippy -D, fmt + check-defs-fmt.sh.
- PROTOCOL 9→10, HASH 47→48 (machine-forced). Coverage 1,083→1,084 clean (60.5%), corpus 1,789→1,792.
- Review: 2 HIGH+1 MED filed; vs cards.sqlite 2 were false positives (docent correct), 1 confirmed
  (thaumatic fabricated ability, fixed+demoted). EF-W-MISS-6 TransformSelf half CLOSED; Battle/Sephiroth split.

## Source findings
- memory/primitives/ef-batch-plan-2026-07-17.md — PB-EF5 section (line ~330), §1 table (EF-W-MISS-6 line 194)
- memory/card-authoring/w-miss-roster-2026-07-17.md — "Card-invokable self-transform effect missing" (line 187)

## COORDINATOR SCOPING DECISIONS (made during recon — constraints for planner/runner)

### DECISION 1 — Ship `Effect::TransformSelf` (the core, highest-yield deliverable)
Unit variant on `Effect` (mirrors `Effect::Meld` at card_definition.rs:2061 — no target field).
Flips `ctx.source` through the SAME machinery as `Command::Transform` / `handle_transform`
(engine.rs:1062). **Reuse, do not fork.** Refactor the core flip out of `handle_transform`
into a shared helper (e.g. `transform_object(state, object_id) -> Result<Vec<GameEvent>>`)
that BOTH the Command path and the new Effect executor call, so CR 701.27c/d/g + daybound/
nightbound + meld-pair guards + PermanentTransformed event + SBA check live in one place.

CR rules the executor MUST honor (verified via MCP):
- **CR 701.27c** — non-DFC → nothing happens (already in handle_transform).
- **CR 701.27d** — back face is instant/sorcery → nothing happens (already there).
- **CR 701.27e/701.28e/701.27f** — **once-per-instruction**: "an activated or triggered
  ability of a permanent … tries to transform it, the permanent does so only if it hasn't
  transformed or converted since the ability was put onto the stack." The task calls this
  "712.8 once-per-instruction" but the actual rule is **701.27f/701.28e**. Guard: track
  whether the source has already transformed during THIS resolving ability/instruction and
  ignore a second TransformSelf. `obj.last_transform_timestamp` already exists — use it
  against the resolving ability's start-of-resolution timestamp, OR a per-execution
  already-transformed set threaded in EffectContext. Planner to pick the cleanest.
- **CR 701.28f / daybound-nightbound** — handle_transform REJECTS daybound/nightbound via
  Command (they only flip via their keyword system). For an on-card TransformSelf effect the
  same rule applies: none of the 11 body-only DFC candidates are daybound/nightbound, so
  TransformSelf may keep the same rejection (or no-op) for them. Planner: confirm no
  candidate is daybound/nightbound; keep the guard.

Wire: new `Effect` variant reaches the SR-8 fingerprint closure (Characteristics→Effect) →
**PROTOCOL 9→10 forced**; Effect is in the GameState hash closure → **HASH 47→48 forced**.
Let the machine gates force both; re-pin PROTOCOL_SCHEMA_FINGERPRINT + sentinel hashes;
append history rows. (Current: PROTOCOL_VERSION=9 @ protocol.rs:113, HASH_SCHEMA_VERSION=47
@ hash.rs:430.)

**TransformNamed: DO NOT ADD.** None of the 11 body-only DFCs transform a *named other*
permanent — every one self-transforms. Speculative variant forbidden (task says "verify
from oracle before adding speculative variants").

### DECISION 2 — CardType::Battle: SPLIT OUT (file seed, do NOT ship)
CR 310 (looked up in full) makes Battle a whole card-type subsystem:
- 310.4b defense counters enter-replacement; 310.6 damage removes defense counters;
- 310.5/508 attackable in combat; 310.8/310.10 protector designation as an SBA (opponent
  chosen at ETB, protector-change SBAs); 310.7 zero-defense → graveyard SBA;
- 310.11b Siege intrinsic "when last defense counter removed, exile + cast transformed."
Shipping a bare `CardType::Battle` enum variant WITHOUT this machinery would produce a
legal-but-wrong Complete def (invariant #9 violation; W6 policy forbids wrong game state).
Invasion of Ikoria // Zilortha stays **blocked / truthfully-marked**. File **OOS-EF5-1**
(dedicated Battle/Siege PB: card type + defense counters + combat attackability + protector
SBA + defeat cast-transformed). Task explicitly permits: "full siege combat semantics beyond
that card may be split out with justification" and "a partial ship of the DFC cohort with
Battle split out is acceptable if justified in a task comment."

### DECISION 3 — Sephiroth "Super Nova": SPLIT OUT (file seed, do NOT ship)
`lookup_card "Sephiroth, Fallen Hero"` = plain legendary creature, NO Super Nova, NO
transform (irrelevant to this PB). The "Super Nova" Sephiroth is the FF-set DFC
(Sephiroth, One-Winged Angel) whose back-face Super Nova is a bespoke keyword action —
its own engine project, not a body-only-DFC flip. File **OOS-EF5-2**. Task: "drop with
justification if it is its own engine project."

## Candidates — the 11 body-only DFCs (chain-verify EACH vs oracle via MCP; demote honestly)
Most have a SECOND blocker beyond TransformSelf (a flip *condition*). Runner must verify the
condition primitive EXISTS before flipping Complete; else mark partial with the real blocker.
Files present: `delver_of_secrets.rs` (currently Complete but flip UNWIRED — needs
TransformSelf + top-of-library-type reveal condition; likely a double-blocker → verify),
`thaumatic_compass.rs`. Missing (author new): bloodline_keeper, docent_of_perfection,
edgar_charmed_groom, fable_of_the_mirror_breaker, grist_voracious_larva, growing_rites_of_itlimoc,
legions_landing, nicol_bolas_the_ravager, westvale_abbey.

Likely-clean with TransformSelf + existing primitives: westvale_abbey ({5},{T},Sac 5
creatures → transform — activated, sac cost), nicol_bolas_the_ravager ({4}{U}{B}{R}: exile+
return transformed — NOTE: "return transformed" is a different mechanism than in-place flip;
verify), growing_rites_of_itlimoc (end-step intervening-if creature count), legions_landing
(attack-with-3+ trigger). Likely double-blocked (stay partial): delver (top-of-library
reveal), fable_of_the_mirror_breaker (Saga chapter-III exile+return transformed), edgar
(dies-return-transformed), grist (enters-as-creature oddity), docent/bloodline (count
intervening-if — verify count-condition primitive exists).

Discounted ship: **~5-8 flips** (task said ~7-9; realistic lower given double-blockers).

## Exhaustive-match reminders (verify `cargo build --workspace` after impl)
- replay-viewer view_model.rs + TUI stack_view.rs: match StackObjectKind/KeywordAbility —
  a new *Effect* variant does NOT touch those, but any new StackObjectKind would. Confirm.
- hash.rs: new Effect discriminant byte.
- effects/mod.rs executor match arm for Effect::TransformSelf.

## Gates
cargo build --workspace; cargo test --all; cargo clippy --all-targets -- -D warnings;
cargo fmt --check + tools/check-defs-fmt.sh.

## Progress (runner)
- [x] Engine Change 1: `Effect::TransformSelf` unit variant added (card_definition.rs, after Meld).
- [x] Engine Change 2: `transform_permanent_in_place` extracted from `handle_transform`
      (engine.rs); `handle_transform` delegates, Command::Transform behavior preserved
      (daybound/nightbound Err still happens before the helper is ever called). Helper
      uses `fizzle_object`/`fizzle_object_mut` (CR 400.7) instead of bare lookups —
      collapsed a redundant re-lookup along the way; bare_lookup_ratchet ceiling for
      engine.rs tightened 24→22.
- [x] Engine Change 3: `EffectContext.source_transformed_this_resolution: bool` added +
      all struct-literal sites updated (effects/mod.rs ::new/::new_with_kicker/2×ForEach
      inner_ctx/condition-delegation literal; rules/abilities.rs activation_condition
      literal — a 6th site not listed in the plan, found by cargo check; 2 test-file
      literals: primitive_pb37.rs, both fixed).
- [x] Engine Change 4: `Effect::TransformSelf` executor arm added in effects/mod.rs
      (after the Meld arm), with the once-per-instruction latch gated on an actual
      `PermanentTransformed` event.
- [x] Engine Change 5: hash discriminant `Effect::TransformSelf => 93u8` added in hash.rs
      (93 confirmed as next-unused by scanning the whole Effect arm — max was 92,
      SetNoMaximumHandSize).
- [x] Engine Change 6: `cargo build --workspace` confirms no other exhaustive match
      needed touching (view_model.rs / stack_view.rs untouched, as predicted).
- [x] Wire bumps: PROTOCOL_VERSION 9→10 + fingerprint re-pinned
      (ec3ccb9e5c1cbdc834c86d6fbbc5d8ee6914e1fe1ef44eeee26d078bbea3d618) + history row +
      FROZEN_HISTORY_PREFIX_DIGEST re-pinned in protocol_schema.rs. HASH_SCHEMA_VERSION
      47→48 + decl/stream fingerprints re-pinned + history row + FROZEN prefix re-pinned
      in hash_schema.rs. All `assert_eq!(HASH_SCHEMA_VERSION, 47)` sentinels bumped to 48
      (bulk sed across 30 test files).
- [x] Card defs: thaumatic_compass flipped Complete; docent_of_perfection authored Complete
      (new file); delver_of_secrets demoted partial (§6a integrity fix).
- [x] Runner discretion (plan §6): bloodline_keeper authored **Complete** (plan's stated 2nd
      blocker was verified FALSE against real oracle text — see deviation note below and
      OOS-EF5-4 in ef-batch-plan-2026-07-17.md §9); growing_rites_of_itlimoc authored
      `partial` (transform half wired via TransformSelf, ETB half truthfully blocked).
      westvale_abbey, legions_landing, grist_voracious_larva, edgar_charmed_groom,
      fable_of_the_mirror_breaker, nicol_bolas_the_ravager left unauthored (real distinct
      blockers, filed as OOS-EF5-3/4).
- [x] Tests: crates/engine/tests/mechanics_m_z/pb_ef5_transform_self.rs (12 tests) + `mod
      pb_ef5_transform_self;` added to mechanics_m_z/main.rs. Non-vacuity spot-check: broke
      the once-per-instruction latch, confirmed test_transform_self_once_per_instruction
      fails, reverted.
- [x] Seeds OOS-EF5-1/2 (coordinator pre-filed) + NEW OOS-EF5-3 (return-transformed
      mechanism, reclassifies grist) + OOS-EF5-4 (DFC flip-condition primitives, 4 cards,
      bloodline_keeper removed from this list) filed into ef-batch-plan-2026-07-17.md §9.
- [x] Final gates: build --workspace clean, test --all 3395/3395 (0 failures), clippy
      --all-targets -D warnings clean, fmt --check clean, check-defs-fmt.sh clean (1792
      defs).

## Deviations from plan (report to coordinator)
1. **bloodline_keeper**: plan §6 table said its 2nd blocker was a "tap N other creatures"
   activation cost (❌ does not exist). Verified via cards.sqlite (scryfall-sourced,
   authoritative) that the real oracle text is "{B}: Transform this creature. Activate only
   if you control five or more Vampires" — a plain mana cost + `activation_condition`
   (Condition::YouControlNOrMoreWithFilter), both already in the DSL. No 2nd blocker exists.
   Authored **Complete** instead of leaving unauthored/partial.
2. **grist_voracious_larva**: plan §6 table described its mechanism as "ETB mill 3; if a
   creature card in GY, transform" (2nd blocker: "creature card in your graveyard"
   Condition). Verified via cards.sqlite the real oracle text uses the SAME
   exile-then-return-transformed mechanism as edgar/fable/nicol_bolas (not an in-place flip
   at all): "...exile Grist, then return it to the battlefield transformed under its owner's
   control." Reclassified from OOS-EF5-4(e) to OOS-EF5-3 (do not author — different
   primitive entirely, matches the coordinator's existing "do not author edgar/fable/bolas"
   guidance).
3. Net effect: 3 Complete (thaumatic_compass, docent_of_perfection, bloodline_keeper) + 1
   demote (delver_of_secrets) + 1 partial (growing_rites_of_itlimoc) = 5 card defs touched/
   authored, vs plan's discounted estimate of "2 Complete + 1 demote + 0-2 partial."
