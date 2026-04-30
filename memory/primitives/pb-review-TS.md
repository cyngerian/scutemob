# Primitive Batch Review: PB-TS — TokenSpec.count u32 → EffectAmount

**Date**: 2026-04-30
**Reviewer**: primitive-impl-reviewer (Opus)
**Branch**: `feat/pb-ts-tokenspeccount-u32-effectamount-dynamic-token-count-pr`
**Commits reviewed**: a48f00e2 (engine), 97b498d3 (hash), 418976c9 (cards), 1bb9926a (tests)
**CR Rules**: 111.1, 111.4, 614.1, 614.1c, 113.7 / 113.7a, 608.2h, 122.1, 122.6, 603.10a, 400.7
**Engine files reviewed**: `cards/card_definition.rs`, `effects/mod.rs`, `state/dungeon.rs`, `state/builder.rs`, `state/hash.rs`, `rules/replacement.rs` (boundary preserved)
**Card defs reviewed**: 4 (Phyrexian Swarmlord, Chasm Skulker, Krenko Mob Boss, Izoni Thousand-Eyed)
**Tests reviewed**: `crates/engine/tests/primitive_pb_ts.rs` (5 new tests; sentinel sweep across 8 files)
**OOS seeds reviewed**: 3 in `memory/primitives/pb-retriage-CC.md`

---

## Verdict: **NEEDS-FIX**

The shape-A migration of `TokenSpec.count` from `u32` to `EffectAmount` is mechanically clean — the
field shape change, dispatch sites in `Effect::CreateToken` / `Effect::CreateTokenAndAttachSource`,
predefined helper updates, dungeon migration, hash bump 13→14, and 8-file sentinel sweep are all
correct. Three of the four authored cards (Phyrexian Swarmlord, Krenko, Izoni) hit the
`PlayerCounterCount` / `PermanentCount` / `CardCount` paths cleanly. However, two issues block
PASS:

1. **HIGH (E1)**: Krenko, Mob Boss has `timing_restriction: Some(TimingRestriction::SorcerySpeed)`,
   but Krenko's oracle text is the unrestricted instant-speed `{T}: Create X 1/1 red Goblin
   creature tokens, where X is the number of Goblins you control.` Setting sorcery-speed
   produces wrong game state (player can't activate Krenko at instant speed). This is a card-def
   bug introduced by the runner.
2. **HIGH (C1)**: Chasm Skulker's WhenDies trigger as authored produces 0 tokens at runtime
   instead of X tokens. The oracle says "create X 1/1 blue Squid creature tokens, where X is the
   number of +1/+1 counters on this creature." After the death trigger fires, ctx.source =
   the post-zone-change graveyard ObjectId, and `move_object_to_zone` (state/mod.rs:420) resets
   `counters: OrdMap::new()`. Dispatching `EffectAmount::CounterCount{Source, PlusOnePlusOne}`
   reads the empty counters map → 0. The runner self-flagged this in test-(d) comment but did
   not (a) escalate to OOS-TS-4, (b) revert chasm_skulker.rs to TODO, or (c) gate the card-def
   on a follow-up LKI-counter-snapshot primitive. Per the W6 policy in CLAUDE.md ("No card is
   authored until its required primitives exist. No TODOs, no partial implementations, no wrong
   game state"), the card cannot ship in this state.

Three LOW findings on stale comments in sentinel-sweep test files. Otherwise the engine work is
solid.

---

## Per-criterion pass/fail

| AC | Description | Status | Notes |
|----|-------------|--------|-------|
| 3724 | Engine primitive landed + 5 tests | **PASS** with caveat | All 5 tests present and compile/pass; test (d) only validates live-source dispatch (does not validate full LKI/Chasm Skulker scenario). |
| 3725 | ≥2 cards re-authored, no-TODO primary mechanic | **FAIL** | Phyrexian Swarmlord, Krenko (*after* Krenko fix), Izoni → 3 valid; Chasm Skulker has wrong game state (count=0 not X). Threshold of 2 met, but C1 HIGH must be cleared. Krenko E1 also needs fix. |
| 3726 | HASH 13→14 + sentinel sweep | **PASS** | HASH_SCHEMA_VERSION = 14 in `state/hash.rs:75`; history entry 14 in lines 67-74; 8 sentinel files updated (5 from plan + 3 expanded by runner). Hash arm dispatches through `EffectAmount::hash_into` (no new arm needed). |
| 3727 | Cargo gates + test count > 2720 | **PASS** | Runner reports 2725 tests, fmt/clippy clean. (Reviewer trusts runner attestation per scope; no re-run required.) |
| 3728 | /review PASS or PASS-WITH-NITS, 0 HIGH/MEDIUM open | **FAIL** | This review opens E1 (HIGH) + C1 (HIGH). Fix-phase round required. |
| 3729 | Plan + review memos committed | **PARTIAL** | Plan committed; this review file pending fix-phase round. |
| 3730 | OOS blockers appended to pb-retriage-CC.md | **PARTIAL** | OOS-TS-1, TS-2, TS-3 appended cleanly. **OOS-TS-4 missing** for Chasm Skulker LKI counter snapshot (the bug surfaced by the runner in test-(d) deviation). |

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| E1 | **HIGH** | `crates/engine/src/cards/defs/krenko_mob_boss.rs:48` | **Krenko activated ability marked sorcery-speed.** Oracle text is `{T}: Create X 1/1 red Goblin creature tokens...` with no "Activate only as a sorcery" restriction (CR 602.5d not present). Setting `timing_restriction: Some(TimingRestriction::SorcerySpeed)` makes Krenko's tap ability sorcery-only — wrong game state, blocks legitimate instant-speed activations. **Fix**: change `timing_restriction: Some(TimingRestriction::SorcerySpeed)` to `timing_restriction: None`. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| C1 | **HIGH** | `chasm_skulker.rs` | **Death trigger creates 0 tokens, not X.** Oracle: "create X 1/1 blue Squid creature tokens, where X is the number of +1/+1 counters on this creature." `EffectAmount::CounterCount{Source, PlusOnePlusOne}` resolved at trigger time reads ctx.source = graveyard new_id; `move_object_to_zone` resets counters to empty (state/mod.rs:420), so resolved count = 0. Toothy precedent in PB-CC re-triage (`pb-retriage-CC.md:80`) and chasm_skulker.rs:33 both claim "counter count is preserved by move_object_to_zone" — these are **aspirationally-wrong code comments** per `memory/conventions.md` "Aspirationally-wrong code comments are correctness hazards." The actual engine dispatches CounterCount → empty → 0. **Fix**: (a) Revert chasm_skulker.rs to TODO; OR (b) Add OOS-TS-4 seed for "WhenDies LKI counter snapshot in PendingTrigger / EffectContext"; AND fix the aspirationally-wrong comment at chasm_skulker.rs:32-35 to "TODO(BASELINE-LKI-04 or OOS-TS-4): WhenDies CounterCount{Source} resolves to 0 tokens because move_object_to_zone resets counters; this card produces wrong game state until a pre-death-counter snapshot mechanism lands." |
| C2 | LOW | `chasm_skulker.rs:32-35` | **Aspirationally-wrong comment.** Comment says "LKI — source is in graveyard but counter count preserved through move_object_to_zone (Toothy precedent)." Actual engine code at `state/mod.rs:420` zeroes counters on zone change; effects/mod.rs:6011-6012 explicitly comments "non-battlefield objects have empty counters maps (counters cease on zone change)." Per conventions.md "Aspirationally-wrong code comments are correctness hazards" — must replace comment with accurate description + tracking-issue cite. **Fix**: After C1 is resolved, ensure the comment matches actual behavior. |

## Other Findings (LOW)

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| L1 | LOW | `crates/engine/tests/pbd_damaged_player_filter.rs:588,594-595` | **Stale comments in sentinel sweep.** Doc comment on line 588 still says "verifies HASH_SCHEMA_VERSION is exactly 11"; inline comment lines 594-595 say "Hash sentinel is bumped to 13 (PB-CC-C-followup ...)." The assertion was correctly updated to 14u8 (line 597) but the surrounding prose was not. **Fix**: Update to "verifies HASH_SCHEMA_VERSION is exactly 14 (PB-TS bump)." |
| L2 | LOW | `crates/engine/tests/pbp_power_of_sacrificed_creature.rs:765-768,779-780` | **Stale comments in sentinel sweep.** Test header line 765-768 says "EffectAmount variants hash distinctly + sentinel 6"; inline comment lines 779-780 says "Assert hash sentinel is exactly 13 (PB-CC-C-followup bump from PB-CC-A's 12...)." Assertion updated to 14u8 correctly. **Fix**: Update header to "sentinel 14" and inline comment to "PB-TS bump from PB-CC-C-followup's 13." |
| L3 | LOW | `crates/engine/tests/pbn_subtype_filtered_triggers.rs:540-553` | **Sentinel-bump history block does not include PB-TS.** The chronological list of bumps stops at "PB-CC-C-followup bumped the sentinel from 12 → 13 (...)" before the assertion `HASH_SCHEMA_VERSION, 14u8`. **Fix**: Append a history line "PB-TS bumped the sentinel from 13 → 14 (TokenSpec.count: u32 → EffectAmount, CR 111.1 / 608.2h)." |
| L4 | LOW | `memory/primitives/pb-retriage-CC.md:401+` | **Missing OOS-TS-4 for Chasm Skulker LKI gap.** The runner appended TS-1, TS-2, TS-3 but did not file the Chasm Skulker / Toothy LKI counter-snapshot primitive that they discovered during test-(d) revision. This is the load-bearing seed that justifies the test deviation. **Fix**: Append OOS-TS-4 to pb-retriage-CC.md with a description like: "Effect::CreateToken (and other LKI-dependent effects) reading EffectAmount::CounterCount{Source} from a graveyard/exile ctx.source resolves to 0 tokens because move_object_to_zone resets counters to empty (state/mod.rs:420). CR 603.10a says leaves-battlefield triggers 'look back in time'; the engine needs a pre-death counter snapshot in PendingTrigger / EffectContext for triggers on the WhenDies / WhenLeavesBattlefield path. Affects: Chasm Skulker (WhenDies token-create, blocked here); Toothy Imaginary Friend (LeavesBattlefield draw — already shipped, also broken); other future LKI-counter cards." |

### Finding Details

#### Finding E1: Krenko sorcery-speed timing_restriction
**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/krenko_mob_boss.rs:48`
**Oracle**: "{T}: Create X 1/1 red Goblin creature tokens, where X is the number of Goblins you control."
**CR Rule**: 602.5d — "Activate only as a sorcery" requires literal text in the oracle; absence
means the ability follows normal instant-speed rules.
**Issue**: The runner's card def has `timing_restriction: Some(TimingRestriction::SorcerySpeed)`,
which makes Krenko's tap activation sorcery-speed-only. Krenko's oracle has no such restriction,
so the def adds an artificial gating that produces wrong game state. Comparison: Birds of
Paradise (`crates/engine/src/cards/defs/birds_of_paradise.rs:19`) and other tap-activated
abilities use `timing_restriction: None`.
**Fix**: Change line 48 from
```rust
timing_restriction: Some(TimingRestriction::SorcerySpeed),
```
to
```rust
timing_restriction: None,
```

#### Finding C1: Chasm Skulker death trigger produces wrong token count
**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/chasm_skulker.rs:36-68`
**Oracle**: "When this creature dies, create X 1/1 blue Squid creature tokens with islandwalk,
where X is the number of +1/+1 counters on this creature."
**CR Rule**: 603.10a (leaves-battlefield triggers look back in time); 122.6 (counter references
during/before zone change).
**Issue**: At runtime, the WhenDies trigger fires after `move_object_to_zone` has moved Chasm
Skulker to the graveyard. The new graveyard `GameObject` has `counters: OrdMap::new()`
(state/mod.rs:420 explicitly resets counters on zone change). When the trigger resolves,
`EffectAmount::CounterCount{Source, PlusOnePlusOne}` calls `resolve_amount` →
`resolve_effect_target_list(EffectTarget::Source)` → ctx.source → graveyard new_id → reads
`state.objects[new_id].counters.get(&PlusOnePlusOne).unwrap_or(&0) = 0`. Result: 0 Squid tokens
created regardless of how many +1/+1 counters Chasm Skulker had on the battlefield. The
docstring at line 33 ("counter count is preserved through move_object_to_zone (Toothy precedent)")
is **aspirationally wrong** — Toothy is also broken, but it's a pre-existing shipped bug that the
PB-CC re-triage doc misclassified as "what works today."

**Validation**: The runner self-reported in `tests/primitive_pb_ts.rs:298-305` that
"`move_object_to_zone` resets counters per CR 400.7 — graveyard object has empty counters" and
revised test (d) to use a live source rather than the planned LKI graveyard scenario. This
acknowledgement is correct, but it was applied to the test only — the card def itself was not
gated, marked TODO, or routed to an OOS seed.

**Verification reference**: `state/mod.rs:420`:
```rust
let mut new_object = GameObject {
    ...
    counters: OrdMap::new(),  // ← counters reset on every zone change
    ...
};
```
And `effects/mod.rs:6011-6012` explicitly states:
```
// CR 122.2: counter check is uniform — non-battlefield objects have
// empty counters maps (counters cease on zone change)...
```

**Fix**: (a) Revert chasm_skulker.rs to a TODO marker citing the LKI-counter primitive gap; OR
(b) Add OOS-TS-4 seed as L4 below requires AND change the comment in chasm_skulker.rs from the
aspirationally-wrong wording to a TODO citation.

If approach (a): replace the WhenDies AbilityDefinition::Triggered with a TODO comment matching
the original (pre-PB-TS) state of the card def for the second ability.

If approach (b): edit chasm_skulker.rs:32-35 to read:
```rust
// When Chasm Skulker dies, create X 1/1 blue Squid creature tokens with islandwalk,
// where X is the number of +1/+1 counters on it.
// TODO(OOS-TS-4): WhenDies CounterCount{Source, PlusOnePlusOne} resolves to 0 tokens
// because move_object_to_zone resets counters to empty (state/mod.rs:420). This card
// produces wrong game state (always 0 Squid tokens) until a pre-death-counter snapshot
// mechanism lands in PendingTrigger / EffectContext per CR 603.10a "leaves-battlefield
// triggers look back in time." Card def kept here for forward-compat; intentionally broken.
```
The reviewer recommends (a) as the cleaner option per W6 policy ("No card is authored until its
required primitives exist"); fix-phase decides.

---

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 111.1 (token creation) | Yes | Yes (tests a-c) | TokenSpec.count → resolved at execution time. |
| 111.4 (token name from subtype) | N/A | N/A | Not changed by PB-TS. |
| 614.1 / 614.1c (token-creation replacement) | Yes (boundary preserved) | Implicitly via existing tests | apply_token_creation_replacement signature unchanged; resolution happens before replacement. |
| 113.7 / 113.7a (LKI source identity) | **No** | **No** | Engine has no pre-death counter snapshot. C1 finding documents the gap. Test (d) deviation explicit. |
| 608.2h (answer determined once at apply) | Yes | Yes (test c — re-execution sees post-mutation count) | resolve_amount called at execution time, not cached. |
| 122.1 / 122.6 (counter timing) | Partial | No | Counters DO reset on zone change in current engine; CR 122.6 implies "while it's on the battlefield" applies, but "looks back in time" (CR 603.10a) is not implemented for counter snapshots. |
| 603.10a (leaves-battlefield looks back in time) | **No** (engine gap) | No | The bug surfaced by C1; OOS-TS-4 seed required. |
| 400.7 (zone change → new object identity) | Yes | Yes (effects/mod.rs:6011-6012 comments) | Engine correctly zeroes counters; this is what makes C1 manifest. |

---

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| `phyrexian_swarmlord.rs` | Yes | 0 | **Yes** | PlayerCounterCount{EachOpponent, Poison} reads PlayerState directly; zone-independent. |
| `chasm_skulker.rs` | Yes (oracle text) | 0 | **NO (C1)** | CounterCount{Source} from graveyard new_id reads empty counters; produces 0 tokens. |
| `krenko_mob_boss.rs` | Yes | 0 | **NO (E1)** | PermanentCount logic correct; timing_restriction wrongly set to SorcerySpeed. |
| `izoni_thousand_eyed.rs` | Yes (primary mechanic only) | 1 (correctly OOS-TS-2) | Yes | Primary ETB mechanic ships; secondary ability legitimately gated on OOS-TS-2. |

---

## Sentinel Sweep Verification

| File | Line | New Sentinel | Stale Comments | Notes |
|------|------|--------------|----------------|-------|
| `tests/primitive_pb_cc_a.rs` | 99 | 14u8 ✓ | None | Renamed to `test_hash_schema_version_after_pb_ts`. |
| `tests/primitive_pb_cc_c_followup.rs` | 394 | 14u8 ✓ | None | Renamed to `test_hash_schema_version_after_pb_ts`. |
| `tests/pbt_up_to_n_targets.rs` | 404 | 14u8 ✓ | None | Renamed to `test_pbt_hash_schema_version_is_14`. |
| `tests/pbt_up_to_n_targets.rs` | 864 | 14u8 ✓ | None | Renamed to `…_sentinel_is_14_regression`. |
| `tests/effect_sacrifice_permanents_filter.rs` | 134 | 14u8 ✓ | None | Renamed to `test_sft_hash_schema_version_is_14`. |
| `tests/pbn_subtype_filtered_triggers.rs` | 555 | 14u8 ✓ | **L3** | History comment block missing PB-TS line. |
| `tests/pbd_damaged_player_filter.rs` | 597 | 14u8 ✓ | **L1** | Header + inline comment still references 11 / 13. |
| `tests/pbp_power_of_sacrificed_creature.rs` | 782 | 14u8 ✓ | **L2** | Header + inline comment still references 6 / 13. |

All 8 sentinel files have correct `assert_eq!(HASH_SCHEMA_VERSION, 14u8, ...)`. Three files have
stale prose around the assertion (LOW findings).

---

## Dispatch Chain Walk Verification

Per `feedback_verify_full_chain.md`, every dispatch site walked:

| Step | Site | Pre-PB-TS | Post-PB-TS | Status |
|------|------|-----------|------------|--------|
| 1 | `cards/card_definition.rs:3116` | `count: u32` | `count: EffectAmount` | ✓ |
| 2 | `cards/card_definition.rs:3158` (Default) | `1u32` | `EffectAmount::Fixed(1)` | ✓ |
| 3 | `treasure_token_spec` (3185) | `count` | `EffectAmount::Fixed(count as i32)` | ✓ |
| 4 | `food_token_spec` (3233) | `count` | `EffectAmount::Fixed(count as i32)` | ✓ |
| 5 | `clue_token_spec` (3282) | `count` | `EffectAmount::Fixed(count as i32)` | ✓ |
| 6 | `blood_token_spec` (3334) | `count` | `EffectAmount::Fixed(count as i32)` | ✓ |
| 7 | `army_token_spec` (3358) | `1` | `EffectAmount::Fixed(1)` | ✓ |
| 8 | `zombie_decayed_token_spec` (3382) | `count` | `EffectAmount::Fixed(count as i32)` | ✓ |
| 9 | `effects/mod.rs:540-590` (CreateToken) | `apply_token_creation_replacement(..., spec.count)` | `let resolved = resolve_amount(state, &spec.count, ctx).max(0) as u32; apply_..._replacement(..., resolved);` | ✓ |
| 10 | `effects/mod.rs:601-622` (CreateTokenAndAttachSource) | `for _ in 0..spec.count` | `let resolved = resolve_amount(state, &spec.count, ctx).max(0) as u32; for _ in 0..resolved` (replacement still NOT called — flagged in OOS-TS-3) | ✓ for resolve, intentional defer for replacement |
| 11 | `state/dungeon.rs:88-185` (5 helpers) | `count: 1` | `EffectAmount::Fixed(1)` | ✓ |
| 12 | `state/dungeon.rs:372` (Muiral's) | `spec.count = 2` | `spec.count = EffectAmount::Fixed(2)` | ✓ |
| 13 | `state/builder.rs:735` (Afterlife) | `count: *n as u32` | `EffectAmount::Fixed(*n as i32)` | ✓ |
| 14 | `state/builder.rs:803` (Living Weapon) | `count: 1` | `EffectAmount::Fixed(1)` | ✓ |
| 15 | `rules/replacement.rs:2603-2637` (apply_token_creation_replacement) | `count: u32` | unchanged | ✓ (boundary preserved) |
| 16 | `state/hash.rs:4308` (TokenSpec hash arm) | `self.count.hash_into(...)` | unchanged (trait dispatch through new EffectAmount::hash_into) | ✓ |
| 17 | `state/hash.rs:75` (HASH_SCHEMA_VERSION) | 13 | 14 + history entry 14 (lines 67-74) | ✓ |
| 18 | 8 sentinel test files | 13u8 | 14u8 | ✓ (3 stale prose comments — L1/L2/L3) |
| 19 | `tests/blood_tokens.rs:812,826` | `assert_eq!(spec.count, 1)` | `assert_eq!(spec.count, EffectAmount::Fixed(1))` | ✓ (per WIP) |
| 20 | `tests/tapped_and_attacking.rs:37` | `count: 2` | `count: EffectAmount::Fixed(2)` | ✓ (per WIP) |
| 21 | `cargo build --workspace` (replay-viewer + TUI exhaustive matches) | n/a | clean per runner | ✓ (trusted) |

**No incidental scope creep observed.** No new variants in `EffectAmount` (the migration uses
existing variants only); no new dispatch arms in `resolve_amount`. The change is purely
mechanical.

---

## Test Verification

| Test | CR Cite | Discriminating? | Notes |
|------|---------|-----------------|-------|
| (a) `test_pb_ts_fixed_count_creates_n_tokens` | CR 111.1 | Yes (4 case loop: 1,3,5,10) | Validates Fixed(N) → N tokens. |
| (b) `test_pb_ts_fixed_zero_creates_no_tokens` | CR 111.1 / 608.2h | Yes | Validates clamp to 0 (catches `u32` truncation regression). |
| (c) `test_pb_ts_permanent_count_scales_with_goblin_count` | CR 111.1 / 608.2h | Yes (cases 0, 2, 4) | Validates PermanentCount linear scaling with battlefield mutations. |
| (d) `test_pb_ts_counter_count_from_live_source` | CR 122.6 | Yes for live-source dispatch only | **Test name accurately describes scope; comment at lines 298-305 acknowledges the LKI scenario is not exercised.** Per conventions.md "Test-validity MEDIUMs are fix-phase HIGHs," this test passes the test-validity bar (the assertion matches the title and CR cite), but the underlying Chasm Skulker card def is not regression-protected at the full-game level. The C1 finding addresses the card-def correctness; this test is independently valid. |
| (e) `test_pb_ts_hash_schema_version_and_token_spec_hash_determinism` | hash infra | Yes (4 sub-checks: sentinel = 14, determinism, Fixed(3)≠Fixed(5), Fixed≠PermanentCount) | Sentinel + variant-discriminant coverage. |

All 5 tests cite CRs in their docstrings, all assertions are discriminating. Test (d) is honest
about its narrower scope.

---

## OOS Seeds Verification

| Seed | Card | Gap | Filed | Notes |
|------|------|-----|-------|-------|
| OOS-TS-1 | Anim Pakal | Non-Gnome attacker trigger filter | ✓ | Lines 403-414 of pb-retriage-CC.md. |
| OOS-TS-2 | Izoni secondary ability | sacrifice-another-creature ActivationCost | ✓ | Lines 416-426 of pb-retriage-CC.md. |
| OOS-TS-3 | Living Weapon doubling | CreateTokenAndAttachSource missing apply_token_creation_replacement | ✓ | Lines 428-439 of pb-retriage-CC.md. |
| **OOS-TS-4** | Chasm Skulker / Toothy / general WhenDies LKI | Pre-death counter snapshot in PendingTrigger or EffectContext for CR 603.10a "looks back in time" semantics | **Missing** | **L4 finding** — must be added before re-review. |

---

## Previous Findings (re-review only)

N/A — first review.

---

## Summary for Fix-Phase

**Required to clear the 0-HIGH-0-MEDIUM gate (AC 3728)**:

1. **E1 fix**: Edit `crates/engine/src/cards/defs/krenko_mob_boss.rs:48` to set
   `timing_restriction: None`.
2. **C1 fix**: Pick option (a) revert chasm_skulker.rs second-ability to TODO; OR option (b)
   keep card def but rewrite the misleading comment to a TODO citation referencing OOS-TS-4.
   Reviewer recommends (a).
3. **L4 fix**: Append OOS-TS-4 to `memory/primitives/pb-retriage-CC.md` documenting the
   pre-death counter snapshot primitive needed for CR 603.10a semantics.

**Recommended (LOW polish)**:
4. L1, L2, L3: Update stale prose comments in pbd / pbp / pbn sentinel-sweep files to
   reference PB-TS bump 13→14 instead of older history.

After E1 + C1 + L4 are addressed, the verdict can flip to PASS-WITH-NITS (with L1/L2/L3 tracked
as deferred LOWs) or PASS (if L1/L2/L3 are also cleaned up).

---

## Re-review (2026-04-30, post fix-phase commit `4fde5d66`)

**Reviewer**: primitive-impl-reviewer (Opus)
**Branch state**: `feat/pb-ts-tokenspeccount-u32-effectamount-dynamic-token-count-pr` @ `4fde5d66` (5 commits total)
**Runner-reported gates**: `cargo build --workspace` clean (8.35s), `cargo test --workspace` 2725 tests passing, `cargo fmt --check` zero diffs, `cargo clippy --all-targets -- -D warnings` zero warnings.

### Verdict: **PASS**

All HIGH findings are resolved; all LOW findings (including the load-bearing L4) are cleared. No
regressions introduced by the fix commit. The PB-TS branch is ready for /review and signal-ready.

### Previous Findings — disposition

| # | Severity | Previous Status | Current Status | Verification |
|---|----------|----------------|----------------|--------------|
| E1 | HIGH | OPEN | **RESOLVED** | `crates/engine/src/cards/defs/krenko_mob_boss.rs:48` now reads `timing_restriction: None,`. Krenko's `{T}` ability is correctly instant-speed per oracle (verified via MCP card lookup — no "Activate only as a sorcery" text). Surrounding card def unchanged; PermanentCount Goblin filter still correct. Mechanically minimal one-line fix. |
| C1 | HIGH | OPEN | **RESOLVED** | `crates/engine/src/cards/defs/chasm_skulker.rs` second ability fully reverted to a TODO comment block (lines 30-36). The `AbilityDefinition::Triggered { trigger_condition: WhenDies, ... }` block is gone. Comment cites `OOS-TS-4`, `state/mod.rs:420`, and `CR 603.10a` "leaves-battlefield triggers look back in time." The aspirationally-wrong "Toothy precedent" / "preserved through move_object_to_zone" wording is removed. The first ability (WheneverYouDrawACard → +1/+1 counter) remains intact. Reviewer recommended option (a) revert; runner chose (a). C2 (LOW companion finding on the misleading comment) is also cleared by the revert. |
| L1 | LOW | OPEN | **RESOLVED** | `crates/engine/tests/pbd_damaged_player_filter.rs:587-599` — doc comment line 588 now says "verifies HASH_SCHEMA_VERSION is exactly 14 (PB-TS bump)"; inline comment lines 594-595 cite "PB-TS bumped TokenSpec.count from u32 → EffectAmount, CR 111.1 / 608.2h"; assertion message correctly mentions "PB-TS bumped HASH_SCHEMA_VERSION 13→14." |
| L2 | LOW | OPEN | **RESOLVED** | `crates/engine/tests/pbp_power_of_sacrificed_creature.rs:765-785` — header line 765 now says "sentinel 14"; inline comment lines 779-780 cite "PB-TS bump from PB-CC-C-followup's 13"; assertion message correctly cites PB-TS. |
| L3 | LOW | OPEN | **RESOLVED** | `crates/engine/tests/pbn_subtype_filtered_triggers.rs:540-558` — history block now contains the new line "PB-TS bumped the sentinel from 13 → 14 (TokenSpec.count: u32 → EffectAmount, CR 111.1 / 608.2h)." (line 553) above the assertion; assertion message correctly cites PB-TS. |
| L4 | LOW (load-bearing) | OPEN | **RESOLVED** | `memory/primitives/pb-retriage-CC.md:441-472` contains the new OOS-TS-4 block. It cites CR 603.10a, CR 113.7a, CR 400.7, CR 122.2; references `state/mod.rs:420` (counters reset) and `effects/mod.rs:6011-6012` (non-battlefield empty counters comment); names two engine paths (a) `EffectAmount::CounterCountAtLastKnownInformation` snapshot in PendingTrigger / EffectContext, (b) preserve counters on graveyard object; explicitly notes Toothy Imaginary Friend as also-broken pre-existing card; documents Chasm Skulker as gated by this seed. The load-bearing seed that justifies the test-(d) deviation is now in place. |

### Per-criterion pass/fail (post-fix)

| AC | Description | Status | Notes |
|----|-------------|--------|-------|
| 3724 | Engine primitive landed + 5 tests | **PASS** | Unchanged from initial review. |
| 3725 | ≥2 cards re-authored, no-TODO primary mechanic | **PASS** | 3 valid ships now: Phyrexian Swarmlord, Krenko (E1 fixed), Izoni primary mechanic. Chasm Skulker correctly reverted to TODO citing OOS-TS-4 (the second ability is gated on the missing primitive; the first ability — counter on draw — still ships and matches oracle, but does not count toward AC 3725 since it was not a token-creation primary mechanic). Threshold of ≥2 met with 3 cards. |
| 3726 | HASH 13→14 + sentinel sweep | **PASS** | Unchanged from initial review. |
| 3727 | Cargo gates + test count > 2720 | **PASS** | 2725 tests post-fix (no test count change — Chasm Skulker scenario was never asserted at full-game level; test (d) already used a live source, so the revert doesn't drop any tests). |
| 3728 | /review PASS or PASS-WITH-NITS, 0 HIGH/MEDIUM open | **PASS** | 0 HIGH / 0 MEDIUM open. All initial-review HIGH findings resolved; all LOW findings (including load-bearing L4) resolved. |
| 3729 | Plan + review memos committed | **PASS** | Plan, review, and re-review section all committed on the branch (or about to be — re-review section is this update). |
| 3730 | OOS blockers appended to pb-retriage-CC.md | **PASS** | OOS-TS-1 through OOS-TS-4 all present and complete. |

### Re-review observations (no new findings)

- **Verified**: Krenko one-line `timing_restriction` change is the only modification to that card def; PermanentCount filter, Cost::Tap, oracle text all preserved.
- **Verified**: Chasm Skulker first ability (WheneverYouDrawACard → AddCounter) is intact; only the second ability (WhenDies token-create) is gone, replaced with TODO comment citing OOS-TS-4. No test in `crates/engine/tests/` referenced the death-trigger token creation, so no test breaks (runner-reported 2725 tests passing confirms this).
- **Verified**: OOS-TS-4 entry is comprehensive and load-bearing — it documents the engine gap with file-line evidence, identifies Toothy as a pre-existing affected card, and proposes two concrete engine paths. This satisfies the W6 stop-and-flag policy ("seed must explain the gap, not just name it").
- **Verified**: All three sentinel-sweep test files have consistent prose ("PB-TS bumped 13 → 14") around the `assert_eq!(HASH_SCHEMA_VERSION, 14u8, ...)` line.
- **No new code paths introduced by the fix commit.** The `4fde5d66` diff is purely (a) a constant change in krenko_mob_boss.rs, (b) a code deletion + comment in chasm_skulker.rs, (c) an OOS seed append in pb-retriage-CC.md, and (d) prose-only edits in three test files. There is no risk of regression in the engine code.

### Next step

The branch is ready for **/review** (PASS-clean, 0 HIGH / 0 MEDIUM open). Coordinator should
proceed to **signal-ready** on ESM task scutemob-16, then merge the branch to main and close the
PB-TS WIP entry.
