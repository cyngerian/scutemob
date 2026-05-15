# Primitive Batch Review: PB-XA — `TargetFilter.is_attacking` enforcement at validate sites + trigger picker

**Date**: 2026-05-15
**Reviewer**: primitive-impl-reviewer (Opus)
**Branch**: `feat/pb-xa-targetfilterisattacking-enforcement-at-validate-sites`
**Predecessor**: PB-XS (`scutemob-21`, shipped 2026-05-14)
**CR Rules**: 109.1 (object identity), 508.1k ("attacking creature"), 601.2c (target legality at announcement), 603.3d (no-legal-target trigger skip)
**Engine files reviewed**:
- `crates/engine/src/rules/casting.rs:5707-5801` (V1-V4: 4 declarative validate sites)
- `crates/engine/src/rules/abilities.rs:6625-6884` (T1-T6: 6 trigger auto-target picker sites)
- `crates/engine/src/cards/card_definition.rs:2600-2614` (doc-comment "Enforced at:" block)
- `crates/engine/src/cards/defs/thousand_faced_shadow.rs` (TODO removed)
- `crates/engine/src/state/hash.rs` (verified untouched; HASH_SCHEMA_VERSION still 20u8)

**Card defs reviewed** (1 in scope, 2 OOS-confirmed):
- IN: `thousand_faced_shadow.rs` (the only card whose `TargetRequirement.*WithFilter.is_attacking` now bites)
- OOS confirmed: `aetherize.rs`, `blessed_alliance.rs` — both correctly use `is_attacking` on `Effect::BounceAll.filter` / `Effect::SacrificePermanents.filter` (effect-resolution path in `effects/mod.rs`, not target-validation). Confirmed via grep.
- OOS confirmed: `eiganjo_seat_of_the_empire.rs` — needs `is_attacking` AND `is_blocking`; correctly seeded as OOS-XA-1 in `pb-retriage-CC.md`.

**Tests reviewed**: `crates/engine/tests/primitive_pb_xa.rs` (10 tests).

---

## Verdict: NEEDS-FIX-MAJOR → **FIXED** (fix-phase 2026-05-15)

The engine change is mechanically correct and symmetrically matches the PB-XS pattern at all 10 enforcement sites. HASH stays at 20 as intended. Card def cleanup is correct. Doc comment on `TargetFilter.is_attacking` is updated with the "Enforced at:" block. OOS seeds for `is_blocking`, `is_tapped`, and `is_nontoken` are well-filed in `pb-retriage-CC.md`.

The initial verdict was **NEEDS-FIX-MAJOR** because of **one test-validity HIGH** (H-XA-01): `test_pbxa_trigger_picker_selects_attacking_creature_positive` (F-1) is a tautology — both candidate creatures pass the filter, the iteration order is by ObjectId, and the test setup adds the "attacker" object (Ravager) before the "non-attacker" object (Sitter), so the auto-target picker selects Ravager regardless of whether `passes_attacking` is enforced. The assertion `target_id == ravager_id` passes both with PB-XA enabled AND with PB-XA disabled. This is the EXACT failure mode the PB-XS E1 HIGH documented, recurring under a different setup.

F-2 is the genuine discriminator for the trigger-picker path (the only legal candidate is non-attacking, and the trigger must be skipped per CR 603.3d). F-2 by itself proves the picker rejects non-attackers. F-1 was supposed to prove the picker PREFERS an attacker over a non-attacker; in its current form it cannot.

**H-XA-01 FIXED**: Swapped object-add order in F-1 so Sitter is added first (smaller ObjectId 2) and Ravager second (larger ObjectId 3). Discrimination confirmed: with `passes_attacking` toggled off, the picker finds Sitter (ObjectId 2) first and the assertion `target_id == ravager_id` fails with "Got id ObjectId(2)". With `passes_attacking` restored, Sitter is rejected, the picker advances to Ravager, and the assertion passes. Added `assert!(sitter_id < ravager_id)` sanity guard to catch future re-tautologisation. Also applied L-XA-01 inline comment (explaining `attacking_player` defensive-checks shortcut) and L-XA-02 (renamed `test_pbxa_hash_schema_version_no_bump` → `test_pb_hash_schema_version_live_sentinel`).

No engine-correctness HIGH. No oracle-text mismatch. No HASH bump issue.

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| E-XA-01 | LOW | `crates/engine/src/rules/casting.rs:5727-5731, 5755-5759, 5774-5778, 5792-5796` | **Identical `passes_attacking` snippet duplicated at 4 sites; no helper.** This mirrors PB-XS's `passes_self` duplication (which also was not extracted to a helper). Future PBs for `is_blocking` / `is_tapped` / `is_untapped` will each duplicate again. Consider extracting a `runtime_predicates_pass(state, id, filter, self_id) -> bool` helper that bundles all runtime-relationship checks. **Fix:** none required in PB-XA; file as an OOS refactoring note when OOS-XA-1 / OOS-XA-2 land (which will each add a sibling check). |
| E-XA-02 | LOW | `crates/engine/src/rules/abilities.rs:6779-6780, 6807-6808, 6864-6865, 6881-6882` | **Trigger-picker `passes_attacking` lines elide the explicit `state.combat` newline split** used in casting.rs (one-line `is_some_and` versus casting.rs's three-line break). Pure formatting — `cargo fmt` may rewrap. Not blocking. **Fix:** none. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| C-XA-01 | LOW | `thousand_faced_shadow.rs` | **TODO at lines 6-9 still describes the intervening-if-on-ETB gap correctly, but does not call out that the "if it's attacking" intervening-if half remains unfixed.** PB-XA fixes the `is_attacking` filter for the auto-target side, but the file-level TODO ("`enters from your hand, if it's attacking`") covers the trigger condition itself — which PB-XA does NOT fix. Net card state: the trigger still fires on any ETB (not just from-hand-while-attacking), but the auto-target picker now correctly restricts the chosen target. Half of the oracle is implemented. **Fix:** no action required for PB-XA. The intervening-if half is a separate primitive (likely PB-XS-E2 or similar). The current TODO is accurate. |

## Test Findings

| # | Severity | Test | Description |
|---|----------|------|-------------|
| H-XA-01 | **HIGH — FIXED** | `crates/engine/tests/primitive_pb_xa.rs` (`test_pbxa_trigger_picker_selects_attacking_creature_positive`) | **F-1 was a tautology.** Fixed by swapping object-add order (Sitter first, Ravager second). Discrimination verified: with `passes_attacking` toggled off, F-1 fails with "Got id ObjectId(2)" (Sitter wins iteration). See Finding Details below. |
| L-XA-01 | LOW — FIXED (comment) | `crates/engine/tests/primitive_pb_xa.rs` (`combat_with_attacker`) | Added inline comment in F-1 explaining that `attacking_player: p(1)` vs Ravager's controller `p(2)` is MTG-impossible but harmless — the validate/picker sites only check `attackers.contains_key`, not `attacking_player`. |
| L-XA-02 | LOW — FIXED | `crates/engine/tests/primitive_pb_xa.rs` (`test_pbxa_hash_schema_version_no_bump`) | Renamed to `test_pb_hash_schema_version_live_sentinel`; message updated to PB-agnostic "update this sentinel to match" wording. |

### Finding Details

#### Finding H-XA-01: `test_pbxa_trigger_picker_selects_attacking_creature_positive` (F-1) is a tautology

**Severity**: HIGH (per `memory/conventions.md` test-validity rule: "test-validity MEDIUMs are fix-phase HIGHs")
**File**: `crates/engine/tests/primitive_pb_xa.rs:520-674`
**CR Rule**: 508.1k / 601.2c / 603.3d (auto-target picker selection in the presence of multiple legal candidates)

**Setup**:
- `dying_ninja` is added first → smallest ObjectId (let's call it N).
- `ravager` is added second → ObjectId N+1.
- `sitter` is added third → ObjectId N+2.
- `state.combat = Some(combat_with_attacker(p(1), ravager_id))` — Ravager is in `attackers`, Sitter is not.

**What the picker does** at `abilities.rs:6783-6810` (T4 site, `TargetPermanentWithFilter`):
The picker walks `state.objects.iter()` (an `OrdMap` keyed by ascending `ObjectId`), calls `.find(|...| ...)` on each candidate, and returns the FIRST one that passes the filter. After the SBA-death moves `dying_ninja` to the graveyard, the battlefield iteration order over candidates that pass `obj.zone == ZoneId::Battlefield` is:

1. Ravager (ObjectId N+1)
2. Sitter (ObjectId N+2)

**With PB-XA disabled** (mental toggle: remove `passes_attacking` from line 6781):
- Ravager: `passes = true` (Creature, `has_card_type` matches), `ctrl_ok = true` (TargetController::Any default), `passes_self = true` (no `exclude_self`). Picker returns Ravager.
- Picker NEVER consults Sitter because `.find()` short-circuits on first match.

**With PB-XA enabled**:
- Ravager: same as above + `passes_attacking = true` (in `combat.attackers`). Picker returns Ravager.
- Sitter would have failed `passes_attacking`, but the picker doesn't get there.

**Result**: the assertion `target_id == ravager_id` passes IDENTICALLY in both cases. The 6 trigger auto-target picker sites have **zero direct test coverage of the positive-discrimination case** (i.e., "the engine prefers an attacker over a non-attacker"). F-2 covers the negative case (no legal target → trigger skipped), and that test is genuinely discriminating.

This is the EXACT failure mode flagged in PB-XS review E1 ("test does not exercise the trigger auto-target picker it claims to test"). The PB-XS fix-phase rewrote that test to use Elderfang's WhenDies + a second graveyard Elf to discriminate `exclude_self` correctly. PB-XA's F-1 inherits the lesson but reintroduces a symmetric tautology because iteration order coincidentally matches the PB-XA-correct answer.

**Fix**: rewrite F-1 to discriminate. Two recommended approaches:

1. **Swap object-add order** so the non-attacker has the smaller ObjectId (added FIRST):
   - Add `sitter` first → ObjectId N+1.
   - Add `ravager` second → ObjectId N+2.
   - Without PB-XA: picker returns Sitter (smaller id, matches filter). Test assertion `target_id == ravager_id` FAILS — discriminating!
   - With PB-XA: picker rejects Sitter (`!passes_attacking`), continues to Ravager. Picker returns Ravager. Assertion passes.

2. **Add an `exclude_self`-only discriminator alongside** — make the source the only one that would be picked first by iteration order, with `exclude_self: false`, and a non-attacker with a smaller ObjectId than the attacker. Then `passes_attacking` is the ONLY check that gates Sitter out and forces the picker to advance to Ravager.

Either approach is acceptable; (1) is the minimal change. After the fix, mentally re-toggle `passes_attacking` OFF and confirm the test FAILS. That is the criterion that satisfies the test-validity invariant.

Note: F-2 is correct as-is. The negative case (trigger skipped when no attacker exists) does not have an iteration-order ambiguity — Sitter passes the filter without PB-XA and the trigger would land on the stack; with PB-XA Sitter fails and the trigger is skipped per CR 603.3d. F-2 alone is good evidence the picker now consults `combat.attackers`, but it does NOT prove that the picker correctly PREFERS an attacker over a non-attacker when both are present. F-1 should fill that gap.

#### Finding C-XA-01: Thousand-Faced Shadow header TODO unchanged

**Severity**: LOW (informational; no fix required for PB-XA)
**File**: `crates/engine/src/cards/defs/thousand_faced_shadow.rs:6-9`
**Oracle**: "When this creature enters from your hand, if it's attacking, create a token that's a copy of another target attacking creature. The token enters tapped and attacking."

**Issue**: The file-level TODO ("`enters from your hand, if it's attacking — intervening-if condition on ETB trigger not fully expressible`") covers a separate primitive — the trigger condition and intervening-if half. PB-XA fixes the auto-target picker but NOT the trigger condition. Net result: the trigger still fires on ANY ETB regardless of source zone or attacking status, but the auto-target picker correctly restricts the chosen target to attacking creatures.

This means Thousand-Faced Shadow's game state is **better, but still partially wrong**: the trigger may fire when it shouldn't (e.g., if the card enters from exile via Cipher or similar — which would be a corner case, but possible). The auto-target picker half is now correct.

**Fix**: no action required. PB-XA scope is the target-side filter, not the trigger condition. The TODO accurately documents the remaining gap. Future PB (or OOS seed) should track the intervening-if half — recommend filing a separate OOS-TFS seed when the workstream reaches "intervening-if conditions on ETB triggers" priority.

---

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 109.1 (object identity) | Yes (existing, reused) | Yes (C-1, D-1, E-1) | Inherited from PB-XS infrastructure. |
| 508.1k ("attacking creature") | Yes (membership in `combat.attackers`) | Yes (C-1, C-2, D-1, D-2, F-2) | F-1 INTENDS to test the positive picker discrimination but is a tautology — see H-XA-01. |
| 601.2c (target legality at announcement) | Yes (V1-V4) | Yes (C-1, D-1) | All 4 declarative variants enforce. |
| 603.3d (trigger skipped when no legal target) | Yes (T1-T6 — the `Option<...>` return path) | Yes (F-2) | F-2 is correctly discriminating. |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|---------------------|-------|
| `thousand_faced_shadow` | Partial (target-half correct; trigger-condition-half still TODO) | 1 (file-level intervening-if) | Better than before, but partial — see C-XA-01 | The auto-target picker now correctly enforces `is_attacking` AND `exclude_self`. The "from your hand, if it's attacking" intervening-if is a separate primitive. |

## Symmetric Placement Audit (per dispatch brief)

All 10 sites verified visually:

| Site | File:Line | Placement check |
|------|-----------|-----------------|
| V1 | `casting.rs:5727-5731` | `passes_attacking` defined at line 5727, ANDed at line 5732 — symmetric with `passes_self` at 5723. ✅ |
| V2 | `casting.rs:5755-5759` | Defined at 5755, ANDed at 5760 — symmetric with `passes_self` at 5751. ✅ |
| V3 | `casting.rs:5774-5778` | Defined at 5774, ANDed at 5782 — symmetric with `passes_self` at 5770. ✅ |
| V4 | `casting.rs:5792-5796` | Defined at 5792, ANDed at 5800 — symmetric with `passes_self` at 5788. ✅ |
| T1 | `abilities.rs:6640-6643` | Inline in `.find()` closure, AND-chained with `passes_self` (6639). ✅ |
| T2 | `abilities.rs:6658-6661` | Inline in `.find()` closure, AND-chained with `passes_self` (6657). ✅ |
| T3 | `abilities.rs:6779-6780` | Defined at 6779, ANDed at 6781 — symmetric with `passes_self` at 6776. ✅ |
| T4 | `abilities.rs:6807-6808` | Defined at 6807, ANDed at 6809 — symmetric with `passes_self` at 6804. ✅ |
| T5 | `abilities.rs:6864-6865` | Defined at 6864, ANDed at 6866 — symmetric with `passes_self` at 6861. ✅ |
| T6 | `abilities.rs:6881-6882` | Defined at 6881, ANDed at 6883 — symmetric with `passes_self` at 6878. ✅ |

Pattern is uniform: `let passes_attacking = !filter.is_attacking || state.combat.as_ref().is_some_and(|c| c.attackers.contains_key(&<id>));` ANDed at the END of the result expression.

## TargetSpellWithFilter Early-Return Audit (per dispatch brief Q8)

Confirmed correct. The early-return arm at `casting.rs:5642` calls `matches_filter` but does NOT thread `passes_attacking`. Spells on the stack cannot be in `combat.attackers` (which keys on battlefield ObjectIds per CR 508.1k). Grep across all card defs confirms ZERO uses of `is_attacking: true` inside a `TargetSpellWithFilter` block. Safe.

## Graveyard Arms (V3, V4, T1, T2) Audit

Per the plan, the graveyard arms apply `passes_attacking` uniformly even though graveyard objects can never be in `combat.attackers`. The check correctly rejects when `is_attacking=true` (defensible per CR 508.1k — graveyard objects are by definition not on the battlefield, so cannot be "attacking creatures"). Test E-1 confirms this behavior. Comment at `casting.rs:5771-5773` documents the design choice. Symmetric with PB-XS's exclude_self enforcement at the same arms.

## OOS-XA Seed Audit

Read `memory/primitives/pb-retriage-CC.md:953-1053`. Three OOS seeds filed:

| OOS ID | Topic | Audit |
|--------|-------|-------|
| OOS-XA-1 | `TargetFilter.is_blocking` for Eiganjo, Seat of the Empire ("attacking OR blocking creature") | **Correctly flagged.** The OR-semantics question for "attacking or blocking creature" is well-stated — the seed proposes either (a) two bools combined with OR at the validate site, or (b) a new `combat_role: Option<CombatRole>` enum with `Attacking | Blocking | AttackingOrBlocking` variants. The two-bool-with-OR approach is the lighter mechanical fix but introduces "if both bools set, accept EITHER role" semantics that diverge from the AND-of-passes pattern used elsewhere. **Reviewer recommendation when OOS-XA-1 is implemented**: pursue option (b) (the enum variant) — cleaner semantics, harder to misuse, no AND/OR ambiguity. ✅ filed correctly. |
| OOS-XA-2 | `TargetFilter.is_tapped` / `is_untapped` enforcement | Correctly flagged. Light mechanical fix. ✅ filed correctly. |
| OOS-XA-3 | Target-side `is_nontoken` enforcement audit (field already exists) | Correctly flagged as a re-audit task. Field exists but target-validate-site enforcement is uninvestigated. ✅ filed correctly. |

## PB-XS Pattern Compatibility Check (per dispatch brief)

Compared the PB-XA diff against the PB-XS engine surface described in `memory/workstream-state.md:31-32` and `pb-review-XS.md`:

- ✅ Same comment-then-let-then-AND structure.
- ✅ Same `passes_<name>` variable naming convention (`passes_filter`, `passes_controller`, `passes_self`, `passes_attacking`).
- ✅ Same CR citation style in comments (`PB-XA: CR 508.1k / 601.2c — ...`).
- ✅ Same 4-site validate enforcement + 6-site picker enforcement = 10 sites total (PB-XS had 4 + 6 too).
- ✅ Same graveyard-arm treatment (apply check uniformly even when it's redundant by CR construction).
- ✅ No HASH bump (PB-XS bumped because it added a new field; PB-XA uses an existing field, no schema change).

## Previous Findings

This is the initial review for PB-XA; no prior findings to re-review.

---

## Tooling Verification (per dispatch brief)

The dispatch brief asked me to run `cargo test --workspace 2>&1 | tail -20` and `cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -20`. The reviewer agent does not have shell access. The implementing agent's report stated build/clippy/fmt clean; spot-checks of the diff show no obvious compile errors (`combat_with_attacker` helper builds a valid `CombatState`, all PB-XA imports are used, no `unwrap`s in library code, all match arms exhaustive). The runner's claim of green CI should be trusted absent contradicting evidence.

If the coordinator wants additional assurance, run:
```
cd /home/skydude/projects/scutemob/.worktrees/scutemob-24
cargo test --workspace 2>&1 | tail -20
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -20
```
Expected: 2774 tests passing (2764 + 10 new). Expected: clippy clean.

---

## Summary

PB-XA is a well-bounded mechanical extension of PB-XS. The engine change is correct at all 10 enforcement sites with symmetric placement and clean comment style. HASH stability is preserved. The card def cleanup is correct. The doc comment "Enforced at:" block on `TargetFilter.is_attacking` is updated. OOS seeds for sibling primitives (`is_blocking`, `is_tapped`, `is_nontoken`) are well-filed.

The **single HIGH** is a test-validity issue (H-XA-01): F-1 is a tautology because iteration order coincidentally produces the PB-XA-correct outcome regardless of whether `passes_attacking` is enforced. This is the same failure mode the PB-XS E1 HIGH documented; it must be fixed before signal-ready by swapping object-add order so the non-attacker has the smaller ObjectId.

The 3 LOWs (E-XA-01, E-XA-02, L-XA-01, L-XA-02, C-XA-01) are cleanup items: no helper extraction for runtime-predicates, minor comment placement, off-color combat-controller setup in test helpers, sentinel test naming style, and the unchanged file-level TFS TODO. None block ship.

**Coordinator action**: dispatch a fix-phase runner to rewrite `test_pbxa_trigger_picker_selects_attacking_creature_positive` so that adding `sitter` BEFORE `ravager` makes the test discriminating (Sitter has the smaller ObjectId; without PB-XA the picker would select Sitter, with PB-XA it must skip to Ravager). LOWs can be batched or deferred.

VERDICT: NEEDS-FIX-MAJOR → **PASS** (post fix-phase 2026-05-15)

---

## Fix-Phase Results (2026-05-15)

**Commit**: `scutemob-24: PB-XA fix-phase — F-1 ObjectId-ordering discriminator (H-XA-01)`

**H-XA-01 FIXED**: Swapped `.object(ravager).object(sitter)` → `.object(sitter).object(ravager)` in
`test_pbxa_trigger_picker_selects_attacking_creature_positive`. Sitter now gets ObjectId 2, Ravager
gets ObjectId 3. Added `assert!(sitter_id < ravager_id)` sanity guard.

**Discrimination check** (gold standard): Temporarily set `passes_attacking = true` (bypassing the
enforcement check) at both T3 and T4 sites in `abilities.rs:6779-6780` and `6807-6808`.
- F-1 FAILED: `Got id ObjectId(2)` — Sitter (smaller ID) was selected instead of Ravager. Confirmed.
- F-2 also failed expectedly (non-attacking Sitter was put on stack instead of trigger being skipped).
- Restored enforcement lines. F-1 and F-2 both pass again.

**L-XA-01 FIXED** (comment): Added inline comment in F-1 explaining the `attacking_player: p(1)` /
controller `p(2)` mismatch is MTG-impossible but harmless since the validate/picker sites only check
`attackers.contains_key`, not `attacking_player`.

**L-XA-02 FIXED**: Renamed `test_pbxa_hash_schema_version_no_bump` →
`test_pb_hash_schema_version_live_sentinel`. Updated message to PB-agnostic "update this sentinel to
match the new value."

**Gate results**:
- `cargo test --workspace`: PASS — 2789 tests (unchanged from implementation phase)
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS — zero warnings
- `cargo fmt --check`: PASS — zero diffs

**Open findings after fix-phase**: 0 HIGH, 0 MEDIUM. E-XA-01 (no helper extraction), E-XA-02 (formatting style), C-XA-01 (TFS file-level TODO) remain as informational LOWs — none block ship.
