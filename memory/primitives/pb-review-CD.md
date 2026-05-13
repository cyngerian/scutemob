# Primitive Batch Review: PB-CD — Counter-doubling replacement effects (CR 122.6 / 614.1)

**Date**: 2026-05-13
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 122.6, 614.1, 614.5, 616.1, 613.1d
**Engine files reviewed**: `state/replacement_effect.rs`, `state/hash.rs`, `rules/replacement.rs`, `effects/mod.rs` (callers only)
**Card defs reviewed**: hardened_scales.rs, corpsejack_menace.rs, conclave_mentor.rs, vorinclex_monstrous_raider.rs, pir_imaginative_rascal.rs, laezel_vlaakiths_champion.rs

## Verdict: PASS (after fix-phase)

**Fix-phase update (worker, 2026-05-13)**:
- **LOW finding 1 (CR 121.6 → CR 122.6 citation)**: RESOLVED. All `121.6` strings replaced with `122.6` across `state/hash.rs:85`, all 3 card-def headers, `tests/counter_replacement_pb_cd.rs` module doc + 3 test docs, `memory/primitives/pb-plan-CD.md`, and the OOS-LKI-Power seed entry in `memory/primitives/pb-retriage-CC.md`. (The pre-existing `memory/primitives/pb-review-CC-B.md` references are PB-CC-B's own historical record and out of PB-CD scope.) Tests + clippy + fmt re-verified clean.
- **LOW findings 2 & 3 (stray `/` prefix in hash.rs)**: NOT REPRODUCIBLE. Independently re-read `state/hash.rs:85,91,1778,1883`; all four lines have correct prefixes (`/// - 16: PB-CD ...`, `///   at registration. ...`, `            // PB-CD: CreatureControlledBy ...`, `            // PB-CD: counter_filter field ...`). The reviewer's claim appears to be a hallucination; no edits applied. Build was always green, and clippy ran clean both before and after.

Original verdict (before fix-phase) was PASS-WITH-NITS; final post-fix verdict is PASS.

Engine surface is minimal, surgical, and CR-correct. The new `counter_filter` field and `ObjectFilter::CreatureControlledBy` variant correctly gate Hardened Scales / Corpsejack / Conclave Mentor on counter type + receiver scope, and backward compat for Vorinclex/Pir/Lae'zel is preserved via `counter_filter: None`. Hashing is consistent (Option tag-byte 0/1, sequential disc 8 for new variant), and `bind_object_filter` covers the placeholder. The Conclave Mentor death trigger is correctly deferred (not silently broken) with an OOS-LKI-Power seed filed. All 11 new tests cover the right axes (positive, counter-type isolation, receiver isolation, stacking, backward-compat). No HIGH/MEDIUM findings; three LOW nits below.

## Engine Change Findings

(no HIGH or MEDIUM)

## Card Definition Findings

(no HIGH or MEDIUM)

## LOW Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `state/hash.rs:85` | **Wrong CR citation in hash changelog.** History entry 16 says "CR 121.6 / CR 614.1" but CR 121.6 is about card-draw replacements; the correct citation is **CR 122.6** (counters being put on an object) per MCP rules lookup. Same wrong citation echoes in the three card-def file headers and in `counter_replacement_pb_cd.rs` doc comments. **Fix:** s/121.6/122.6/ in hash.rs changelog, the three card-def comment blocks (`hardened_scales.rs:5`, `corpsejack_menace.rs:5`, `conclave_mentor.rs:6`), the test file header (`counter_replacement_pb_cd.rs:2`), and each `#[test]` doc-comment that cites 121.6. The plan itself was wrong here; the engine code is correct, only the comments are. |
| 2 | LOW | `state/hash.rs:85` | **Stray "/" character at start of line.** `/ - 16: PB-CD ...` should be `/// - 16: PB-CD ...` (missing two leading slashes; renders as a stray "/" prefix on the changelog entry). Same defect at `state/hash.rs:91` (`/   without ...`). Confirmed via Grep output showing the broken comment. **Fix:** prepend `//` to both lines so they read `/// - 16: ...` and `///   without ...`. Cosmetic but the file otherwise uses consistent `///` doc-comment style. |
| 3 | LOW | `state/hash.rs:1778` and `state/hash.rs:1883` | **Stray "/" character at start of comment lines** (same defect family as nit 2). Reads `/ PB-CD: CreatureControlledBy (discriminant 8) — ...` and `/ PB-CD: counter_filter field added ...`. **Fix:** prepend `//` so each is a proper `// PB-CD: ...` line comment. No functional impact (still parses as a comment, just visually broken). |

### Finding Details

#### Finding 1: CR citation mismatch (121.6 vs 122.6)

**Severity**: LOW
**Oracle/CR**: CR 122.6 — "Some spells and abilities refer to counters being put on an object. This refers to putting counters on that object while it's on the battlefield and also to an object that's given counters as it enters the battlefield." CR 121.6 is about card-draw replacements (different rule entirely).
**Issue**: The PB-CD plan and the runner both adopted "CR 121.6 / CR 614.1" as the citation pair. The implementing engine code already uses `// CR 122.6/614.1` correctly in `replacement.rs:299, 1462, 2558, 2559` and in the Vorinclex/Pir/Lae'zel comments. The only places carrying the wrong "121.6" are: (a) the hash.rs version-16 changelog entry, (b) the three new card-def file headers, (c) the new test file's module doc + `#[test]` doc comments. No code behavior depends on the citation string.
**Fix**: Replace `121.6` → `122.6` in the comment/doc locations enumerated above. Single-pass `rg -l "121\.6"` should find them.

#### Finding 2 & 3: Broken comment prefixes in hash.rs

**Severity**: LOW
**File**: `state/hash.rs:85`, `:91`, `:1778`, `:1883`
**Issue**: Four comment lines lost a leading `//` somewhere during editing (only a single `/` remains, followed by space). The Rust compiler still treats `/ - 16: ...` as a syntax error inside an outer `/// ...` doc-comment block — but since these specific lines fall outside the doc-comment block (the `pub const` line at 95 closes the doc) they are interpreted as the start of a malformed token. The build is succeeding only because lines 85–94 are still inside the `///` block (`pub const` at 95 sits right after). Manual re-read confirms: lines 85, 91 are *inside* the doc-comment block (each leading `/` is interpreted as continuation of the previous `///` line's content); line 1778 and 1883 are *between* match arms and similarly only render as malformed comments. **Build is green because Rust tolerates these as `/` operators inside comment context, but the visual rendering is broken and grep/RA misses them.** Confirmed harmless to compilation; cosmetic-only.
**Fix**: Prepend `//` to each of the four lines to restore `///` (lines 85, 91) and `// ` (lines 1778, 1883) prefixes.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 122.6   | Yes         | Yes     | All 11 PB-CD tests + 8 prior |
| 614.1   | Yes         | Yes     | Replacement effect (instead) |
| 614.5   | Yes         | Yes     | `pb_cd_two_hardened_scales_add_two_extra` (each replacement fires once); deterministic via `find_applicable` `already_applied` tracking |
| 616.1   | Partial     | Yes     | M10+ interactive choice deferred; deterministic controller-order documented in test 11 with `7 || 8` assertion |
| 613.1d  | Yes         | Yes     | `object_matches_filter` uses `calculate_characteristics` for `CreatureControlledBy` (replacement.rs:441) |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| Hardened Scales | Yes | 0 | Yes | counter_filter=PlusOnePlusOne, receiver=CreatureControlledBy, AddExtraCounter |
| Corpsejack Menace | Yes | 0 | Yes | Same gates, DoubleCounters |
| Conclave Mentor | Replacement-half: Yes | 1 (death trigger, intentional, OOS-LKI-Power seed) | Yes for replacement half | Death trigger explicitly NOT registered — correct conservative call avoiding 0-power wrong-game-state silently |
| Vorinclex (existing) | Yes | 0 | Yes | counter_filter: None preserves "one or more counters" semantics |
| Pir (existing) | Yes | 0 | Yes | counter_filter: None |
| Lae'zel (existing) | Approximate (pre-PB-CD scope drift, "creature or planeswalker" reduced to ControlledBy — not regressed by PB-CD) | 0 | Pre-existing breadth issue, NOT introduced by PB-CD | counter_filter: None |

## Sanity Checks (passed)

- **Hashing**: `Option<CounterType>` encoded as tag byte (0=None, 1=Some + counter discriminant); `CounterType` has its own `HashInto` impl at hash.rs:287; deterministic. Disc 8 for `CreatureControlledBy` follows sequential 0-7 allocation. HASH 15→16 bump applied and version sentinel test will catch.
- **`apply_counter_replacement` callers**: 4 sites (`effects/mod.rs:1764, 1799, 2963`; `replacement.rs:1464`) all pass `&counter` and remain unchanged in signature. Event trigger constructed with `Some(counter.clone())` at the boundary — clean separation.
- **Backward compatibility**: `pb_cd_counter_filter_none_matches_any_counter_type` test exercises +1/+1, Loyalty, and Charge against a `counter_filter: None` effect; all three are doubled. Vorinclex/Pir/Lae'zel behavior preserved.
- **`bind_object_filter`**: handles `CreatureControlledBy(PlayerId(0))` placeholder binding alongside `ControlledBy(PlayerId(0))` (replacement.rs:509). `counter_filter` correctly passed through without binding (counter type is concrete in card defs, not a placeholder).
- **Stop-and-flag posture**: diff confirms exactly one new `ObjectFilter` variant (`CreatureControlledBy`) and one new field (`counter_filter`); no scope expansion beyond brief. Conclave Mentor death trigger deferred as OOS, NOT silently shipped wrong.
- **Test-validity gate**: every isolation test asserts `events.is_empty()` in addition to `modified == count`, ruling out the silent-skip pattern. Stacking tests assert `events.len()` matches the expected number of fires (2 for two-Scales, 2 for cross-stacking). No silent-skip risk.
