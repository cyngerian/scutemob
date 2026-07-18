# Primitive WIP: PB-EF9 — `EffectDuration::WhileYouControlSource` (EF-W-PB2-5)

batch: EF9
title: Add `EffectDuration::WhileYouControlSource` — a "for as long as you control [source]" continuous-effect duration (CR 611.2b/c) that differs from `WhileSourceOnBattlefield` ONLY under gain-control of the source. "You" = the effect creator's controller AT CREATION. Once the duration's condition stops being met the effect ends permanently and NEVER resumes even if control returns (CR 611.2c). Wire the placeholder-resolution to `ctx.controller` (mirror `UntilYourNextTurn(PlayerId)`), add its expiry/reversion in the continuous-effect machinery, flip olivia_voldaren's `{3}{B}{B}` gain-control half, sweep for similar borrow-a-creature effects.
task: scutemob-110
branch: feat/pb-ef9-effectdurationwhileyoucontrolsource-ef-w-pb2-5
started: 2026-07-18
phase: fix-complete  # review: memory/primitives/pb-review-EF9.md (0 HIGH, 1 MEDIUM, 2 LOW). MEDIUM fixed this session (see "Fix phase" section below): expire_while_you_control_source_effects moved to top-of-loop in check_and_apply_sbas so control reverts within the SAME call that kills the source via SBA (704.5g), not lagging to the next call. New regression test added + mutation-tested. LOW #1 (SingleObject-only reversion) addressed with a documentation NOTE per review's own "no fix required, note for next author" instruction. LOW #2 (olivia target-Vampire filter) needs no change per review. All gates re-run green; HASH/PROTOCOL unchanged (52/14) as expected — this was an internal call-site move, not a wire change. implement phase COMPLETE 2026-07-18 (this session). Plan: memory/primitives/pb-plan-EF9.md. All 8 engine changes + 4 card-def fixes + 10 tests (mutation-tested) + version bumps (PROTOCOL 13→14, HASH 51→52) done. Key finding: NO control-reversion existed in engine before this PB (WhileSourceOnBattlefield gain-control never reverted either); built imperatively via expire_while_you_control_source_effects + recompute_object_controller. OOS-EF9-1 filed for the latent UntilEndOfTurn/WhileSourceOnBattlefield never-reverts gap (deferred, not fixed here). Ready for collection.

## Fix phase (2026-07-18, this session) — applied review findings from `pb-review-EF9.md`

**MEDIUM (sba.rs:75) — FIXED.** Control reversion lagged when the source left the
battlefield via a state-based action (the canonical exit for Olivia/Silumgar: dying
to combat damage or 0 toughness). The old placement called
`expire_while_you_control_source_effects` exactly once, *before* the SBA fixpoint
loop; a source dying *inside* that loop (SBA 704.5g) was not observed until the
*next* `check_and_apply_sbas` call — an observably-wrong priority window where the
borrower still controlled a permanent it should have lost.

**Fix applied exactly as specified**: moved the
`expire_while_you_control_source_effects(state)` call from once-pre-loop to the
**top of every loop iteration** in `crates/engine/src/rules/sba.rs::check_and_apply_sbas`
(before `apply_sbas_once`). Rewrote the CR comment block in place to explain: (a) why
top-of-loop placement is required (the source-death SBA fires *inside* the loop,
one+ passes after a pre-loop-only call would have run), (b) why it cannot loop
forever (the expiry pass is one-shot/idempotent — CR 611.2c permanently removes
ended effects, never re-adds them — so a steady-state iteration is a no-op), and (c)
that termination still keys exclusively off `apply_sbas_once`'s returned events,
never off this call.

**New regression test**: `test_while_you_control_source_reverts_when_source_dies_via_sba`
(CR 611.2b/704.5g) in `crates/engine/tests/primitives/pb_ef9_while_you_control_source.rs`.
Source enters already marked with lethal damage (`.with_damage(2)` on a 2/2), so it
dies to SBA 704.5g on the very first pass inside a single `check_and_apply_sbas`
call; asserts (1) the source actually died within that call (test invariant), (2)
the borrowed creature reverts to its owner within that SAME call, (3) the
`WhileYouControlSource` effect is gone. **Proven non-vacuous**: temporarily reverted
`sba.rs` to the pre-loop-only placement (pre-fix behavior), re-ran
`cargo test --test primitives pb_ef9` — exactly this one new test failed (9/10
passed, the new one FAILED with the borrowed creature still under p1 instead of
reverting to p2), all 9 pre-existing tests stayed green. Restored the fix;
`md5sum crates/engine/src/rules/sba.rs` after restore = `5bd445c0ec00b0e381590d3a7022feaf`,
byte-identical to the pre-mutation snapshot taken right after the fix was first
applied. Full suite re-run: 10/10 pb_ef9 tests green.

**LOW #1 (layers.rs:1754, no fix required) — documentation NOTE added.** Per the
review's own instruction ("Fix: none required now; note for the next author"), added
a `// NOTE:` at the `EffectFilter::SingleObject` collection site in
`expire_while_you_control_source_effects` explaining that only `SingleObject` reverts
control today (true for every current card, all authored via `GainControl`), and
that a future `WhileYouControlSource` effect built via `ApplyContinuousEffect` with a
broader filter (`AllPermanents`, `CreaturesYouControl`, etc.) would be removed in
Step 2 but NOT reverted in Step 3 — flagging where to extend if that ever happens.

**LOW #2 (olivia_voldaren.rs "target Vampire" → creature filter) — no change.** Per
the review, this is practically correct (all Vampire permanents are creatures) and
needs no fix.

**Version bumps: NONE.** This is a pure internal call-site move (same function,
same effect, different loop position) — no field/variant/serde shape changed on any
wire type or `GameState`-reachable type. Confirmed by re-running the full suite: no
`protocol_schema`/`hash_schema` test reddened. `PROTOCOL_VERSION` stays **14**,
`HASH_SCHEMA_VERSION` stays **52**.

**Gates re-run after the fix — all green:**
- `cargo build --workspace` — clean
- `cargo test --all` — 0 failures, **3437** total passed (3436 baseline + 1 new
  regression test)
- `cargo clippy --all-targets -- -D warnings` — clean
- `cargo fmt --all -- --check` — clean
- `tools/check-defs-fmt.sh` — clean, 1792 defs checked (unaffected by this fix; no
  card def touched)

## Engine changes — ALL DONE, `cargo build --workspace` clean
- [x] Change 1: `EffectDuration::WhileYouControlSource(PlayerId)` variant — `crates/card-types/src/state/continuous_effect.rs`
- [x] Change 2: `is_effect_active` arm (layers.rs) — returns `true` always; never a live control check
- [x] Change 3: placeholder→controller resolution in `Effect::GainControl` AND `Effect::ApplyContinuousEffect` (effects/mod.rs); `Effect::ExchangeControl` left as a `// NOTE:` comment only (no card needs it)
- [x] Change 4: `expire_while_you_control_source_effects` (layers.rs, new fn, after the other `expire_*` fns)
- [x] Change 5: `recompute_object_controller` (layers.rs, new fn) — reapplies remaining active SetController effects in timestamp order
- [x] Change 6: call site — `sba.rs::check_and_apply_sbas`, called once pre-loop
- [x] Change 7: hashing — `hash.rs`, discriminant 5
- [x] Change 8: exhaustive match sweep — `cargo build --workspace` found ONE extra site the plan's table missed: `crates/engine/src/rules/replacement.rs` L199-242 (a second, separate `is_effect_active` for *replacement* effects, not continuous effects). Added the same `=> true` arm there with a comment. replay-viewer/tui/simulator confirmed non-exhaustive (grep — simulator's only reference uses `matches!`, not an exhaustive match).

## Card def fixes — ALL DONE (`cargo check -p mtg-card-defs` clean)
- [x] olivia_voldaren.rs → `Completeness::Complete` (primary flip)
- [x] dragonlord_silumgar.rs → `known_wrong` → `Completeness::Complete`; stale "control correctly reverts when Silumgar leaves" note replaced with accurate text
- [x] roil_elemental.rs → stays `partial`. Verdict: the "you may" optional wrapper is **NOT expressible** — confirmed `Effect::MayPayThenEffect` requires a real `Cost` (not a bare optional wrapper) and `Effect::MayPayOrElse` is a gated stub (SR-33) that never offers the choice; no `is_optional` flag on `AbilityDefinition::Triggered`. Did NOT author a mandatory GainControl. Note rewritten to name the optional-wrapper primitive gap as the sole residual blocker.
- [x] kellogg_dangerous_mind.rs → stays `partial`. TODO + note refreshed: only remaining blocker is the Treasure-sacrifice-count cost (`Cost::Sacrifice` has no count field); deleted the stale "duration ARE available" aspirational wording, replaced with "available as of PB-EF9."
- [x] TODO sweep (mandatory): `grep -rln "WhileYouControlSource\|for as long as you control" crates/card-defs/src/defs/*.rs` → 8 hits: the 4 above + 4 false positives (mirage_phalanx, silverblade_paladin, tandem_lookout, wingcrafter — all Soulbond `WhilePaired`, unrelated). Broader sweep on `gain control` + TODO/ENGINE-BLOCKED found 3 more control-stealing blockers on DIFFERENT primitives (captivating_vampire: count-of-subtype tap cost; hellkite_tyrant: "all artifacts" plural-target gap; emrakul_the_promised_end: gain control of a PLAYER not a permanent). None reference this primitive. No additional forced adds.

## Tests — 9 new tests, `crates/engine/tests/primitives/pb_ef9_while_you_control_source.rs`, registered in `primitives/main.rs`
All 9 pass. Mutation-tested (non-vacuity proof) for every guard: for each, applied the
mutation, ran `cargo test --test primitives pb_ef9`, confirmed exactly the expected
tests failed (and no others), then reverted and confirmed `md5sum
crates/engine/src/rules/layers.rs` was byte-identical to the pre-mutation snapshot.
1. Controller-mismatch check disabled (`ended` forced `false` when source exists) →
   5 tests failed exactly as expected (`ends_when_opponent_gains_source`,
   `does_not_resume`, `multiplayer`, `olivia_stolen`, DECOY's own sanity assertion).
   `ends_when_source_leaves`, `phase_out`, `silumgar`, `olivia_dies` correctly still
   passed (they don't exercise this branch).
2. Permanent-removal step skipped (effect stays in `continuous_effects` after ending,
   so `recompute_object_controller` still finds it "active" and never reverts) →
   8 of 9 tests failed; only `survives_source_phase_out` stayed green (nothing "ends"
   in that scenario, so there is nothing to remove and no reversion to observe).
3. CR 702.26e phase-out guard: reinstated the `WhileSourceOnBattlefield`-style
   `is_phased_in()` check → exactly 1 test failed
   (`survives_source_phase_out`), all other 8 stayed green.
4. CR 400.7 source-gone handling: `unwrap_or(true)` → `unwrap_or(false)` → exactly 3
   tests failed (`ends_when_source_leaves`, `olivia_dies`, `silumgar`), all other 6
   stayed green.
`layers.rs` md5 before any mutation: `6c3c44cbc29d994eea637d8c8f46a2f8`; after all four
mutate/restore cycles: identical.

## Version bumps — MACHINE-FORCED, all values read from failing-gate output (never guessed)
- `PROTOCOL_VERSION`: 13 → 14. `PROTOCOL_SCHEMA_FINGERPRINT` set to
  `b94f90e1c6d7f4193385489f6f6d541dbb764534eab09593584f99361ea828d7` (read from
  `protocol_schema_fingerprint_is_pinned` failure). Appended `ProtocolEpoch { version: 14, .. }`
  to `PROTOCOL_HISTORY` (never edited an existing row). `protocol_version_sentinel` → 14.
  `FROZEN_HISTORY_PREFIX_DIGEST` re-pinned to `648f47c35743fb50f826ba32ab25cabc1bdb73471eb6f7ca8c7b31593c96e343`
  (read from `frozen_prefix_is_pinned` failure, since version 13 joined the frozen prefix).
- `HASH_SCHEMA_VERSION`: 51 → 52. Appended `HashSchemaEpoch { version: 52, .. }` to
  `HASH_SCHEMA_HISTORY` with `decl_fingerprint =
  0e8ef019079eb88c574f8cb08cdb0e421b0c319a8ec2b942ae94694c58126fee` and `stream_fingerprint =
  d90e8be93a121620e014738c8d1139a5198e31d25de40d89e56faba55f33421e` (both read from
  `declaration_fingerprint_is_pinned` / `stream_fingerprint_is_pinned` failures — used a
  placeholder-row-first technique to surface them, since `current_epoch()` panics outright if
  no row exists yet for the new version). `hash_schema_version_sentinel` → 52.
  `FROZEN_HISTORY_PREFIX_DIGEST` (hash_schema.rs) re-pinned to
  `c034c53bf920e7d39227566883d56351ef5ed0a7881a7417ac1fae8be89adccd`.
- Scattered sentinels: grepped whole tree for `HASH_SCHEMA_VERSION, 51u8` (32 files, 33
  occurrences incl. one file with 2) and `PROTOCOL_VERSION, 13` (1 file,
  `pb_ef7_modal_activated.rs`) — all bumped to `52u8` / `14` respectively. Relative-math
  usages (`PROTOCOL_VERSION - 1`, `HASH_SCHEMA_VERSION.wrapping_sub(1)` in
  `protocol_roundtrip.rs`) do not need editing — verified they reference the constant, not a
  literal.
- `docs/mtg-engine-protocol-versioning.md`: checked, has no per-version row table (no
  "N→N+1"-style rows) — the plan's conditional step does not apply, no edit made.
- Two unplanned test-file fixes surfaced by the full suite (both consequences of legitimate
  card flips / new engine code, not bugs):
  - `bare_lookup_ratchet` ceiling for `src/rules/layers.rs`: 51 → 54 (3 new NONSWALLOW-shaped
    bare `.objects.get[_mut]` reads in the new expire/recompute functions, each with a
    documented CR 400.7 fizzle rationale in the ratchet's own comment).
  - `completeness_deviation_scan::the_marker_detector_is_not_vacuous` threshold: `marked >= 700`
    → `marked >= 690` (olivia + silumgar's flip to Complete legitimately dropped the corpus's
    non-Complete count from 701 to 699, crossing the old floor; lowered with the same margin
    the old threshold kept, comment cites this PB).

## Gates — ALL GREEN
- `cargo check -p mtg-engine` clean
- `cargo build --workspace` clean
- `cargo test --all` clean (0 failures across every target; card_defs_fmt + check-defs-fmt.sh
  both pass)
- `cargo clippy --all-targets -- -D warnings` clean
- `cargo fmt --all -- --check` clean (ran `cargo fmt --all` once to fix 3 wrapping diffs in the
  new test file, then re-verified `--check` exits 0)
- `tools/check-defs-fmt.sh` clean (also ran with `--fix` once, which reformatted
  kellogg_dangerous_mind.rs and roil_elemental.rs — line-wrap only, no semantic change,
  verified by re-reading both files)
- `python3 tools/authoring-report.py` regenerated: clean 1,091/1,792 (60.9%) →
  1,093/1,792 (61.0%), +2 exactly matching the two flips

## OOS-EF9-1 (filed, NOT fixed here)
Latent gap: `Effect::GainControl` with `WhileSourceOnBattlefield` (pre-flip usage elsewhere)
and with `UntilEndOfTurn` never reverts `obj.controller` when the continuous effect is
removed — the effect disappears from `continuous_effects` but control silently persists.
Roster: sarkhan_vol.rs, zealous_conscripts.rs, karrthus_tyrant_of_jund.rs (all `UntilEndOfTurn`
gain-control, shipped Complete). Also: `test_gain_control_until_eot_expires` in
primitive_pb32.rs is vacuous with respect to control reversion — it asserts the effect is
removed but never asserts `obj.controller` reverts. The `recompute_object_controller` helper
built in this PB (layers.rs) is exactly what a follow-up PB would wire into
`expire_end_of_turn_effects`'s removal path to close this. Deferred — out of scope here.

## Source findings
- memory/primitives/ef-batch-plan-2026-07-17.md — PB-EF9 section (~line 479); §1 table (EF-W-PB2-5, line 192)
- memory/card-authoring/w-pb2-roster-2026-07-17.md — EF-W-PB2-5 (line 101)
- memory/card-authoring/w-pb2-engine-findings-2026-07-17.md — EF-W-PB2-5 (line 137, full writeup)

## Recon (coordinator/worker, 2026-07-18) — hand to planner
- `EffectDuration` enum: `crates/card-types/src/state/continuous_effect.rs` L44-68. Variants:
  WhileSourceOnBattlefield, UntilEndOfTurn, Indefinite, WhilePaired(ObjectId,ObjectId),
  UntilYourNextTurn(PlayerId). Note the existing payload-carrying precedent.
- `is_effect_active` (the live activity check per layer recompute): `crates/engine/src/rules/layers.rs:501`.
  `WhileSourceOnBattlefield` arm at L503: source on battlefield + phased in.
- `Effect::GainControl` handler: `crates/engine/src/effects/mod.rs:5356`. Builds a Layer-2
  `SetController(ctx.controller)` ContinuousEffect with `source: Some(ctx.source)`, `duration: *duration`,
  and imperatively writes `obj.controller = controller`. NOTE: olivia is authored via GainControl, NOT
  ApplyContinuousEffect.
- `UntilYourNextTurn` placeholder→controller resolution pattern: `effects/mod.rs:3147-3157`
  (card DSL uses `PlayerId(0)` placeholder; resolved to `ctx.controller` at effect creation). MIRROR THIS.
- Hashing of `EffectDuration`: `crates/engine/src/state/hash.rs:1903`.
- **OPEN QUESTION for planner** (verify via RA/CR): control reversion mechanism. `calculate_characteristics`
  treats `LayerModification::SetController` as a NO-OP (`layers.rs:1094`, "controller lives on GameObject");
  `obj.controller` is written imperatively in the GainControl handler and there is NO layer-based
  control-reconcile loop found. So determine EXACTLY how/where an ending `WhileSourceOnBattlefield`
  control effect reverts `obj.controller` today (search around `is_effect_active` callers, SBA, priority,
  move_object_to_zone cleanup, `state/mod.rs:1555` GC). The new duration's expiry must revert control the
  SAME way, and must PERMANENTLY remove the effect so it can never resume.
- olivia_voldaren def: `crates/card-defs/src/defs/olivia_voldaren.rs` — currently `partial`, `{3}{B}{B}`
  uses `EffectDuration::WhileSourceOnBattlefield`.

## Design constraints (must hold)
1. "You" is captured at creation (ctx.controller), stored in the variant payload (placeholder `PlayerId(0)`
   in DSL → resolved at creation), analogous to `UntilYourNextTurn(PlayerId)`.
2. Ends when creator no longer controls source (source left battlefield OR control of source changed away).
3. **Never resumes**: a pure live re-eval in `is_effect_active` is INSUFFICIENT/WRONG on its own —
   if control returns, live re-eval would revive it. Must permanently remove the effect the first time
   the condition fails (an `expire_*`-style pass), reverting the borrowed permanent's control.
4. Wire change (new enum variant reachable from the SR-8 protocol closure via Effect/Characteristics) →
   bump PROTOCOL_VERSION and HASH_SCHEMA_VERSION only if machine-forced; justify.
