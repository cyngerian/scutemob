# Primitive WIP: PB-OS4 — return-transformed / enters-transformed as a NEW object (OOS-EF5-3)

batch: OS4
task: scutemob-130
branch: feat/pb-os4-return-transformed-enters-transformed-as-a-new-object
started: 2026-07-19
phase: review

Plan: `memory/primitives/pb-plan-OS4.md`. Review: `memory/primitives/pb-review-OS4.md`.

## Brief (THE PLAN IS `memory/primitives/oos-retriage-plan-2026-07-18.md` §3 PB-OS4; canonical finding `memory/primitives/ef-batch-plan-2026-07-17.md` §9 OOS-EF5-3)

CAPABILITY, **highest yield** of the capability group. First capability dispatch off the PB-OS
queue (correctness group PB-OS1..OS3 complete).

A permanent that dies / is exiled and RETURNS to the battlefield already on its back face is a
**NEW object entering transformed** (CR 712.18) — a fundamentally different mechanism than the
in-place `Effect::TransformSelf` that PB-EF5 shipped (which keeps the same `ObjectId`). CR 400.7:
the returned permanent is a new object; the old `ObjectId` is dead; auras/counters do NOT carry;
"when this dies" triggers reference the OLD object.

## Fix shape (per plan §3 / OOS-EF5-3)
A `ReturnTransformed`/`enters_transformed` flag on the zone-change/return effect
(`Effect::MoveZone` or a dedicated `Effect::ReturnTransformed`) threaded through the return path so
the new object enters with **back-face** characteristics, layer-resolved; PLUS Saga-chapter
integration for Fable. **New wire type → PROTOCOL bump** (one bump for the whole PB, machine-forced
by SR-8 gates; justify in close-out — do NOT fight the gate).

## Candidates (4 — EACH verified vs oracle text via MCP BEFORE impl; PB-EF5 caught 2 mis-filed cards this way; honest yield ~2-3)
- **edgar_charmed_groom** — dies → delayed trigger returns it to the battlefield transformed at the
  next end step.
- **fable_of_the_mirror_breaker** — Saga chapter III: exile, return transformed (riskiest — Saga
  integration).
- **nicol_bolas_the_ravager** — `{4}{U}{B}{R}`: exile self, return transformed at next end step.
- **grist_voracious_larva** — re-verified via MCP (plan table was stale): "Whenever Grist or another
  creature you control enters, if it entered from your graveyard or you cast it from your graveyard,
  you may pay {G}. If you do, exile Grist, then return it to the battlefield transformed under its
  owner's control." — identical return-transformed mechanism, NOT a TransformSelf case.

Discounted ship **~2-3** of 4 — honest yield beats forced flips; a card with a distinct 2nd blocker
stays truthfully marked with the blocker NAMED (PB-EF5 precedent).

## Mandatory tests
- **New-object identity (CR 400.7)**: old ObjectId dead; auras/counters do NOT carry; a "when this
  dies" trigger references the OLD object. Pin CR 400.7 by test.
- **Enters-transformed characteristics (CR 712.18)**: the returned object has back-face
  characteristics, **layer-resolved** (calculate_characteristics, not raw def read).
- **Delayed-trigger timing**: return happens at the **next end step**, not immediately (edgar,
  nicol_bolas).
- **Saga chapter ordering** for Fable if shipped.
- **Decoys must fail on exactly the field under test** (SR-34/36 probe-by-execution).

## Wire bump (AC 5040)
Single PROTOCOL bump (with HASH if forced) for the whole PB, justified in close-out per SR-8;
update the sentinel tests + history rows. Do NOT fight the gate.

## Close-out (AC 5041)
Close (or honestly narrow) **OOS-EF5-3** in `oos-retriage-plan-2026-07-18.md` §3 PB-OS4 (SHIPPED
banner + table strike) and `ef-batch-plan-2026-07-17.md` §9 (CLOSED banner). Update shipped-card
header comments. Update this WIP status. Non-shipped cards keep their real named blocker.

## Plan outcome (2026-07-19)
- **Design**: 2 dedicated unit `Effect` variants — `ExileSourceAndReturnTransformed` (immediate; fable ch. III) + `ReturnSourceToBattlefieldTransformedNextEndStep` (delayed; edgar dies→end-step) — plus new `DelayedTriggerAction::ReturnFromGraveyardToBattlefieldTransformed`. Reuses craft return path (engine.rs:1422-1441), ExileWithDelayedReturn idiom, SagaChapter. NOT a flag on MoveZone (blast radius).
- **Honest yield ~2**: fable_of_the_mirror_breaker + edgar_charmed_groom → Complete. nicol_bolas_the_ravager + grist_voracious_larva **STAY OUT** — named blocker: planeswalker back face + `CardFace` has no `starting_loyalty` → 0-loyalty PW dies to SBA 704.5i. File follow-up seed **OOS-OS4-1**. grist also needs entered-from-graveyard trigger condition.
- **Brief correction**: nicol_bolas returns IMMEDIATELY (not next end step) — moot (stays out) but recorded.
- **Wire**: PROTOCOL 18→19, HASH 55→56 machine-forced (3 new enum variants in SR-8 closure). Effect hash discriminants 94/95. `DelayedTriggerAction` matched at 4 hash.rs sites — all need new arm.

## Steps
- [x] 1. Plan — primitive-impl-planner → pb-plan-OS4.md (verified 4 cards; design + Saga; PROTOCOL/HASH bump justified)
- [x] 2. Implement engine change (return-transformed threading; Saga integration for Fable) — primitive-impl-runner
  - Added `Effect::ExileSourceAndReturnTransformed` + `Effect::ReturnSourceToBattlefieldTransformedNextEndStep` (card_definition.rs, near TransformSelf); `DelayedTriggerAction::ReturnFromGraveyardToBattlefieldTransformed` (stubs.rs). Immediate executor arm (effects/mod.rs, mirrors craft return path) + delayed registration arm; dispatch arm in resolution.rs (mirrors ReturnFromExileToBattlefield). Hash: Effect discriminants 94/95; DelayedTriggerAction local discriminant 5 at all 4 sites. `cargo check --workspace` clean (no TUI/replay-viewer match gaps — confirmed, matches plan's Change 9 prediction).
- [x] 3. Flip card defs (only genuinely-correct ones; others keep named blocker)
  - **edgar_charmed_groom.rs** (new file) → `Completeness::Complete`. Front: static +1/+1 anthem (Vampires) + WhenDies→`ReturnSourceToBattlefieldTransformed`. Back (Edgar Markov's Coffin): upkeep token+counter+conditional `TransformSelf` loop. Every clause expressible.
  - **fable_of_the_mirror_breaker.rs** (new file) → `Completeness::partial(...)`, NOT Complete. Chapter III (`Effect::ExileSourceAndReturnTransformed`, the PB-OS4 primitive) is fully wired and real. Two residuals, both unrelated to PB-OS4's primitive and both genuinely inexpressible today (not gated stubs): (a) chapter I's Goblin Shaman token needs its own attached triggered ability ("whenever this token attacks, create a Treasure") — `TokenSpec` has no `triggered_abilities` field; (b) chapter II ("discard up to two, if you do draw that many") has no DSL primitive (`DiscardCards` has no bounded player-choice count; `WheelHand` only disposes of the whole hand). Back face's activated ability also inherits `kiki_jiki_mirror_breaker.rs`'s existing known-wrong gap (no `TargetFilter` "nonlegendary" exclusion) — noted in the partial message rather than causing a separate KnownWrong tier.
  - **nicol_bolas_the_ravager**, **grist_voracious_larva** — left UNAUTHORED (no def file), matching PB-EF5 precedent. Named blocker: planeswalker back face + `CardFace` has no `starting_loyalty` (OOS-OS4-1).
  - `tools/check-defs-fmt.sh --fix` run once (Fable's back-face `types_sub(...)` line needed wrapping); full defs corpus re-verified clean afterward.
- [x] 4. Tests (identity CR 400.7, characteristics CR 712.18, timing, Saga)
  - New module `crates/engine/tests/mechanics_m_z/pb_os4_return_transformed.rs`, `mod` line added to `mechanics_m_z/main.rs`. 14 tests: new-object identity, counters-do-not-carry (decoy TransformSelf), aura-falls-off, back-face-characteristics (decoy front name), non-DFC-stays-in-exile (decoy DFC returns), delayed timing (decoy not-on-battlefield-before-end-step), delayed new-object, immediate-no-exile (decoy no ObjectExiled event — for the 3rd effect), Saga ch.III no-sacrifice (decoy plain-effect ch.III IS sacrificed), fable + edgar card-def integration (edgar via full check_triggers/flush_pending_triggers/stack-resolution dispatch path), fable-marked-partial / edgar-marked-complete / nicol-grist-not-complete integrity guards. 14/14 passing.
- [x] 5. PROTOCOL/HASH bump + sentinel/history rows updated (2 commits — see divergence note below)
  - **DIVERGENCE (STOP-and-flag, proceeded per "follow the text" authorization)**: at authoring time, `cards.sqlite` oracle text for Edgar, Charmed Groom showed NO "at the beginning of the next end step" clause — "When Edgar, Charmed Groom dies, return it to the battlefield transformed under its owner's control" resolves IMMEDIATELY on the WhenDies trigger, with no exile step either (unlike Fable). Neither of the plan's two effects fit this shape. Added a THIRD unit effect, `Effect::ReturnSourceToBattlefieldTransformed` (immediate, no exile — mirrors Persist/Undying's `Effect::MoveZone{Source->Battlefield}` idiom but also flips to back face + registers statics/ETB). This makes `ReturnSourceToBattlefieldTransformedNextEndStep` (the delayed variant) currently unused by any roster card — kept anyway since it's a real, distinct CR 603.7 primitive, tested standalone per plan §8.
  - PROTOCOL_VERSION 18→19 (`rules/protocol.rs`), fingerprint `1d0dc7b8d5ea44129090b873826d798e84dd7698d1b2170214b66d65d2543e05`, FROZEN_HISTORY_PREFIX_DIGEST re-pinned to `427628738bef89b1a939590242978b532810bfbaea7f44b8d07ce6275c07b6c1`.
  - HASH_SCHEMA_VERSION 55→56 (`state/hash.rs`), decl `d8752059bb71f8c104ab76caf4995055dd9bdd2e8fe5c298e79cb3dbecaa2b98`, stream `46da56438f4951cb7b3eb76ed35fa966ffa738b6449eb611459c77359ba455ee`, FROZEN_HISTORY_PREFIX_DIGEST re-pinned to `4f1b8eba2e9cfb60cf8e7aed5d56f774b09d959352fc911af9016b0b39ac2bb2`.
  - All values copied verbatim from the failing gate tests' expected output (never hand-computed), per SR-8/SR-17 doctrine.
  - Bulk-updated ~35 scattered sentinel assertions (`HASH_SCHEMA_VERSION, 55u8` → `56u8`; `PROTOCOL_VERSION, 18` → `19`) across `crates/engine/tests/`.
  - SR-25 `bare_lookup_ratchet` gate caught 2 new bare `.objects.get(` lookups in the new `effects/mod.rs` executor + 1 in the new `resolution.rs` dispatch arm; converted to `fizzle_object`/`expect_object` (no ceiling bump needed — kept at 107/102).
  - `cargo test -p mtg-engine --test core`: 424 passed, 0 failed.
- [x] 6. Review — primitive-impl-reviewer → pb-review-OS4.md. **1 HIGH + 2 MEDIUM.**
  - **HIGH (H1)**: the return path fires FRONT-face abilities, not back-face. `register_static_continuous_effects` (replacement.rs:2057) + `queue_carddef_etb_triggers` (:1415) + the upkeep trigger scan (turn_actions.rs:277) all iterate front `def.abilities` with NO `is_transformed`/`back_face` branch (only keywords read back_face, layers.rs:116). Verified directly. Consequence: Edgar Markov's Coffin upkeep loop never fires AND Edgar's front Vampire anthem wrongly re-registers onto the Coffin (WRONG game state) → Edgar cannot be Complete OR Partial. This is a general transform-machinery gap (likely also affects PB-EF5 in-place TransformSelf Complete markers). = **OOS-OS4-2** (own PB).
  - **MED (double bump)**: PB did TWO wire bumps (18→19→20 / 55→56→57). AC 5040 requires ONE. Collapse to a single 18→19 / 55→56 (intermediate versions never left this branch).
  - **MED (unused variant)**: `ReturnSourceToBattlefieldTransformedNextEndStep` + `DelayedTriggerAction` unused → REMOVE.
- **SCOPE DECISION (coordinator/user, 2026-07-19): SHIP NARROWED.** Face-aware ability gathering (OOS-OS4-2) is out of scope (its own PB, touches general transform machinery, may change shipped TransformSelf behavior — STOP-and-flag). Reduce PB-OS4 to what ships honestly:
  - Keep ONLY `Effect::ExileSourceAndReturnTransformed` (used by Fable ch. III — correct for new-object identity + back-face characteristics + Saga no-sacrifice; no wrong state).
  - **REMOVE** `Effect::ReturnSourceToBattlefieldTransformed` (edgar-immediate, no shipping card) + `Effect::ReturnSourceToBattlefieldTransformedNextEndStep` + `DelayedTriggerAction::ReturnFromGraveyardToBattlefieldTransformed` (unused).
  - **Fable** → stays Partial (ch. III wired = real primitive usage; ch. I/II inexpressible; back-face Reflection activated ability blocked by OOS-OS4-2 — name it). No wrong state (Reflection inert, front Saga abilities are Triggered not Static so nothing wrongly registers).
  - **Edgar** → UN-AUTHOR (delete edgar_charmed_groom.rs) — would emit wrong state (front anthem on Coffin) until OOS-OS4-2. Blocker: OOS-OS4-2.
  - **nicol_bolas, grist** → stay unauthored. Blocker: OOS-OS4-1 (loyalty); grist also needs entered-from-graveyard trigger condition.
  - **Single wire bump**: PROTOCOL 18→19, HASH 55→56 (one effect variant only — re-pin fingerprints FRESH from failing gates after removals; the runner's earlier 19/20/56/57 digests are STALE, closure changed).
  - **Honest yield: 0 Complete flips, 1 Partial with real primitive usage.** The return-transformed *mechanism* ships (AC 5038: new object + back-face characteristics layer-resolved, tested). File OOS-OS4-1 + OOS-OS4-2. Narrow OOS-EF5-3 (NOT fully closed — Complete flips blocked by OOS-OS4-1/2).
- [x] 6b. Fix pass (SHIP NARROWED) — primitive-impl-runner. Removed 2 unused effect variants + `DelayedTriggerAction::ReturnFromGraveyardToBattlefieldTransformed`; kept ONLY `Effect::ExileSourceAndReturnTransformed` (hash discriminant 94). Deleted `edgar_charmed_groom.rs`. Fable stays partial (ch. III wired; blockers named incl. OOS-OS4-2). Single wire bump PROTOCOL 18→19 (fingerprint `14d2b0d4...`), HASH 55→56 (decl `0be185c3...`, stream `46da5643...`); intermediate v20/v57 rows removed; digests re-pinned FRESH from failing gates. Test module pruned 14→9 (removed delayed/immediate-no-exile/edgar-integration; integrity guard widened to all 3 unauthored cards).
- [x] 7. Green gates — ALL GREEN: `cargo build --workspace`, `cargo test --all` (0 failures), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`, `tools/check-defs-fmt.sh` (1799 defs). Independently spot-verified PROTOCOL=19 / HASH=56, no code refs to removed variants, edgar gone / fable present, no v20/v57 rows.
- [x] 8. Close-out docs — ef-batch-plan §9 OOS-EF5-3 NARROWED banner (not fully closed); §13 filed **OOS-OS4-1** (planeswalker-back starting loyalty) + **OOS-OS4-2** (face-aware back-face ability gathering, likely also affects PB-EF5 TransformSelf markers). oos-retriage-plan §PB-OS4 SHIPPED-NARROWED banner + queue table strike.
phase: close-out (pending /review + Completion Sequence)

## Final outcome (2026-07-19)
- **Shipped**: `Effect::ExileSourceAndReturnTransformed` — return-transformed as a NEW object (CR 400.7 + 712.18), back-face characteristics layer-resolved, no counters/auras carried, Saga 714.4 no-sacrifice. PROTOCOL 18→19, HASH 55→56 (single bump). 9 tests in `mechanics_m_z/pb_os4_return_transformed.rs`.
- **Yield (honest)**: 0 Complete flips; `fable_of_the_mirror_breaker` **partial** with real ch. III primitive usage. edgar/nicol/grist unauthored with named blockers.
- **New seeds**: OOS-OS4-1 (loyalty), OOS-OS4-2 (face-aware ability gathering). OOS-EF5-3 narrowed, not fully closed.
- **AC status**: 5038 ✓ (primitive ships end-to-end, CR 400.7 pinned), 5039 ✓ (all 4 verified vs oracle; honest yield, non-shipped carry real blockers), 5040 ✓ (single PROTOCOL/HASH bump, sentinels/history updated), 5041 ✓ (all gates green; /review pending; OOS-EF5-3 honestly narrowed in plan + source docs).
