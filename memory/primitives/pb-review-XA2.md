# Primitive Batch Review: PB-XA2 — `TargetFilter.is_blocking` + `is_tapped` + `is_untapped` runtime predicates

**Date**: 2026-05-15
**Reviewer**: primitive-impl-reviewer (Opus, independent post-merge review)
**Commit reviewed**: `9eca57af` (merged via `f3905b62` to `main`; runner self-collected, no pre-merge independent review)
**Predecessor**: PB-XA (`scutemob-24`, shipped 2026-05-15) + PB-XS (`scutemob-21`, 2026-05-14)
**CR Rules**: 109.1 (object identity), 508.1k ("attacking creature"), 509.1 / 509.1c ("blocking creature"), 601.2c (target legality at announcement), 603.3d (no-legal-target trigger skip), 701.20a (tap), 701.21a (untap)

**Engine files reviewed**:
- `crates/engine/src/cards/card_definition.rs:2600-2659` (`is_attacking` doc updated + 3 new fields with doc comments)
- `crates/engine/src/state/combat.rs:115-124` (`CombatState::is_blocking` helper)
- `crates/engine/src/state/hash.rs:146-157, 4341-4388` (HASH_SCHEMA_VERSION 21→22 + history block + 3 hash arms)
- `crates/engine/src/rules/casting.rs:5707-5868` (V1-V4: 4 declarative validate sites)
- `crates/engine/src/rules/abilities.rs:6625-6944` (T1-T6: 6 trigger auto-target picker sites)
- `crates/engine/src/cards/defs/eiganjo_seat_of_the_empire.rs` (TODO removed, filter switched)

**Test file reviewed**: `crates/engine/tests/primitive_pb_xa2.rs` (17 tests, ~1378 LOC)
**Canary sentinel bumps reviewed**: 16 existing test files (all 21u8 → 22u8 with PB-XA2-cited messages)
**OOS seeds reviewed**: `memory/primitives/pb-retriage-CC.md:1141-1196` (OOS-XA2-1..5)

**Card defs reviewed** (1 in scope):
- `eiganjo_seat_of_the_empire.rs`: switched `TargetCreature` → `TargetCreatureWithFilter { is_attacking: true, is_blocking: true, .. }`. Oracle verified independently via `mcp__mtg-rules__lookup_card`: "Channel — {2}{W}, Discard this card: It deals 4 damage to target attacking or blocking creature. This ability costs {1} less to activate for each legendary creature you control." Matches def.

---

## Verdict: PASS

PB-XA2 is a clean, mechanical 3-predicate extension of PB-XA with no HIGH or MEDIUM findings. All scrutinized risk areas (G-1 discriminator validity, Path A closure refactor, OR-semantics 10-site symmetric placement, HASH bump completeness, Eiganjo oracle match, graveyard-arm semantics, doc-comment cross-references, OOS seed quality) pass independent verification. The runner correctly internalized H-XA-01's lesson and shipped the G-1 discriminator as a genuine positive test (Sitter added first with smaller ObjectId; sanity guard `assert!(sitter_id < defender_id)` present; mental-toggle check documented as performed). The graveyard-arm `is_untapped=true` degenerate edge case is locked in via test H-1c and called out in V3 source comments — no live correctness risk.

3 LOW informational findings noted below; none warrant a follow-up fix commit. The merged main is ship-clean. Coordinator action: none. LOWs may be batched into a future cleanup pass or addressed when OOS-XA2-5 (helper extraction) lands.

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| E-XA2-01 | LOW | `crates/engine/src/cards/card_definition.rs:2652-2659` | **`is_untapped` doc comment is briefer than `is_tapped`'s** — uses "Enforced at: same sites as `is_tapped`" rather than restating the 4 + 6 site list. Not a correctness issue; only a minor template-uniformity asymmetry. **Fix:** none required; consider expansion when OOS-XA2-5 helper extraction lands and rewrites the surrounding docs anyway. |
| E-XA2-02 | LOW | `crates/engine/src/rules/abilities.rs:6815-6817, 6851-6853, 6916-6918, 6941-6943` | **T3-T6 use single-line `passes_tapped` / `passes_untapped` definitions** (vs. casting.rs's 2-line indented split). Matches the PB-XA `passes_attacking` style precedent in abilities.rs but diverges from casting.rs formatting. Pure style; `cargo fmt --check` already passes per the runner. **Fix:** none. |
| E-XA2-03 | LOW | `crates/engine/src/rules/casting.rs:5810-5815` + `tests/primitive_pb_xa2.rs:1314-1377` | **Graveyard arm `is_untapped=true` ACCEPTS graveyard candidates** (degenerate-but-documented edge case). The comment in V3 calls this out and test H-1c locks the behavior in. Symmetric with V4 (which references V3's note). Semantically meaningless for any legitimate card. The right long-term fix (if a real card ever needs graveyard-side tap state) is a dedicated graveyard-target predicate, not the runtime field. **Fix:** none required; future audit may revisit if a card needs zone-specific tap semantics. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| (none) | — | `eiganjo_seat_of_the_empire.rs` | **Clean.** Oracle text matches def exactly. TODO removed. Filter is `TargetCreatureWithFilter { is_attacking: true, is_blocking: true, ..Default::default() }` — exercises the (T,T) OR-semantics branch of `passes_combat_role`. The `activated_ability_cost_reductions[(0, ...)]` referencing index 0 is correct: mana ability `{T}: Add {W}` is a mana ability (lives in `mana_abilities`, not `activated_abilities`), so Channel is activated-ability index 0. Pre-existing and not touched by PB-XA2. |

## Test Findings

| # | Severity | Test | Description |
|---|----------|------|-------------|
| (none) | — | `primitive_pb_xa2.rs::test_pbxa2_trigger_picker_selects_blocking_creature_positive` (G-1) | **G-1 is a genuine positive discriminator.** Object-add order: `dying_scout` (will die), `sitter` (non-blocker, smaller ObjectId), `attacker`, `defender` (blocker, larger ObjectId). Sanity guard `assert!(sitter_id < defender_id)` at line 982-989. Mental-toggle reasoning verified: with `passes_combat_role` removed at T4, picker would short-circuit on Sitter (smaller ObjectId, passes type filter, no other restriction); assertion `target_id == defender_id` would fail with `Got id ObjectId(<sitter_id>)`. With enforcement on, Sitter fails `is_blocking` gate; picker advances to Defender. No tautology. H-XA-01 lesson correctly applied. |

### Finding Details

#### Finding E-XA2-01: `is_untapped` doc comment terseness

**Severity**: LOW (style only)
**File**: `crates/engine/src/cards/card_definition.rs:2652-2659`
**Issue**: `is_tapped`'s doc comment (lines 2638-2651) restates the "Enforced at: 4 validate + 6 picker" template with full path enumeration ("same 4 variants as `is_attacking`"). `is_untapped`'s doc (lines 2652-2659) shortens to "same sites as `is_tapped`". A reader navigating in isolation must hop through `is_tapped` first to find the site list. The PB-XA `is_attacking` doc (lines 2600-2618) is the gold standard for verbosity.
**Fix**: none required; cosmetic. Consider expanding `is_untapped`'s doc to mirror `is_tapped`'s site enumeration when OOS-XA2-5 (helper extraction) lands and rewrites surrounding context. Alternatively, accept "same as is_tapped" as a deliberate antiduplication. No impact on correctness.

#### Finding E-XA2-02: T3-T6 single-line `passes_tapped` / `passes_untapped` formatting

**Severity**: LOW (formatting style only)
**File**: `crates/engine/src/rules/abilities.rs:6815-6817, 6851-6853, 6916-6918, 6941-6943`
**Issue**: At T3-T6, `passes_tapped` and `passes_untapped` are single-line definitions (`let passes_tapped = !f.is_tapped || obj.status.tapped;`). In casting.rs (V1-V4), the same expressions span two lines with indentation (`let passes_tapped = / !filter.is_tapped || state.objects.get(&id).is_some_and(|o| o.status.tapped);`). This mirrors the PB-XA `passes_attacking` precedent in abilities.rs (which also uses tighter formatting) but is asymmetric across the two files. The discrepancy arises because T3-T6 read directly from `obj.status.tapped` (no `state.objects.get(...)` indirection), so the body fits on one line; V1-V4 must do the lookup explicitly.
**Fix**: none. The formatting is internally consistent (per-file convention) and `cargo fmt --check` passes per the runner's gate-check claim. Pure style.

#### Finding E-XA2-03: Graveyard arm `is_untapped=true` degenerate ACCEPTS behavior

**Severity**: LOW (documented edge case, not a correctness bug)
**File**: `crates/engine/src/rules/casting.rs:5810-5815, 5844-5846`; `tests/primitive_pb_xa2.rs:1312-1377`
**Issue**: For V3 (`TargetCardInYourGraveyard`) and V4 (`TargetCardInGraveyard`), the predicate reads `state.objects.get(&id).is_some_and(|o| !o.status.tapped)`. Graveyard objects default to `status.tapped = false`, so `!status.tapped = true`, so `passes_untapped = true` for any graveyard candidate that matches the other filters. This means setting `is_untapped=true` on a graveyard target filter ACCEPTS all otherwise-matching candidates, while `is_tapped=true` REJECTS them (the symmetric case). Test H-1c locks this in. Comments at lines 5810-5815 document the design choice and reference V3 from V4 (lines 5844-5846).
**Fix**: none required. No card legitimately uses `is_tapped`/`is_untapped` on a graveyard target (CR 110.5 — tapped is a battlefield-only concept). If a future card needs zone-specific tap semantics, the right approach is a new `TargetFilter.requires_battlefield_zone` predicate or a `TargetCardInYourGraveyardWithTapState` requirement, not retrofitting the runtime field. Filed implicitly under OOS-XA2-4 (CombatRole refactor) as a future cleanup if scope expands. Behavior is locked in via H-1c so any future regression triggers a test failure.

---

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 109.1 (object identity) | Yes (inherited via PB-XS `self_id` plumbing) | Yes (C-1, D-1, E-1 reject paths) | Inherited from PB-XS infrastructure. |
| 508.1k (attacking creature) | Yes (`combat.attackers.contains_key`) | Yes (F-1: OR-semantics attacker-arm) | Pre-existing from PB-XA; reused in `passes_combat_role`. |
| 509.1 / 509.1c (blocking creature) | Yes (`combat.is_blocking(id)` via new helper) | Yes (C-1, C-2, F-2, F-3, G-1, G-2) | New `CombatState::is_blocking` helper at `combat.rs:115-124`. Distinct from `is_blocked(attacker_id)`. |
| 601.2c (target legality at announcement) | Yes (V1-V4 in casting.rs) | Yes (C-1, C-2, D-1, D-2, E-1, E-2, F-1, F-2, F-3) | All 4 declarative variants enforce all 3 new fields. |
| 603.3d (trigger skipped when no legal target) | Yes (T1-T6 in abilities.rs — `Option<...>` return path) | Yes (G-2) | Picker returns None → trigger not put on stack. |
| 701.20a (tap) | Yes (`o.status.tapped`) | Yes (D-1, D-2) | Runtime field; not a Characteristics property. |
| 701.21a (untap) | Yes (`!o.status.tapped`) | Yes (E-1, E-2) | Symmetric with 701.20a. |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|---------------------|-------|
| `eiganjo_seat_of_the_empire` | YES (verified via MCP `lookup_card`) | 0 (TODO removed) | YES | Filter uses `(is_attacking=true, is_blocking=true)` → exercises (T,T) OR-semantics branch. Channel cost-reduction filter unchanged (correctly on the cost side, not target side). |

## Symmetric Placement Audit (per dispatch brief)

All 10 sites verified visually:

| Site | File:Line | (T,T) branch uses OR? | Pattern |
|------|-----------|-----------------------|---------|
| V1 | `casting.rs:5728-5751` | YES (line 5738: `\|\|`) | `let passes_combat_role` + `let passes_tapped` + `let passes_untapped` + AND chain |
| V2 | `casting.rs:5772-5795` | YES (line 5782: `\|\|`) | Same |
| V3 | `casting.rs:5816-5837` | YES (line 5826: `\|\|`) | Same + design-quirk comment for graveyard |
| V4 | `casting.rs:5847-5868` | YES (line 5857: `\|\|`) | Same + V3 cross-reference |
| T1 | `abilities.rs:6634-6658` | YES (line 6641: `\|\|`) | Path A inline closure: `role_ok`/`tapped_ok`/`untapped_ok` lets above `.find()`; `combat_ref` captured outside |
| T2 | `abilities.rs:6673-6688` | YES (line 6677: `\|\|`) | Same; `combat_ref2` captured outside |
| T3 | `abilities.rs:6806-6817` | YES (line 6810: `\|\|`) | T3 TargetCreatureWithFilter top-level |
| T4 | `abilities.rs:6842-6853` | YES (line 6846: `\|\|`) | T4 TargetPermanentWithFilter top-level |
| T5 | `abilities.rs:6907-6918` | YES (line 6911: `\|\|`) | T5 UpToN inner TargetCreatureWithFilter |
| T6 | `abilities.rs:6932-6943` | YES (line 6936: `\|\|`) | T6 UpToN inner TargetPermanentWithFilter |

Grep confirmation: all 10 `(true, true) => ... is_some_and(|c| ... || c.is_blocking(...))` sites identified. Zero AND-typo sites. `passes_tapped` count: 8 in abilities.rs + 9 in casting.rs (4 sites × 2 = 8 + 1 in V3 design-quirk comment; matches expected). `passes_untapped` count: 8 + 8 = 16 (sites use it 1× for definition + 1× for AND chain at each of 8 sites). T1/T2 use `tapped_ok`/`untapped_ok` instead (Path A refactor).

## Path A Closure Refactor Audit (T1/T2)

`abilities.rs:6625-6663` (T1, TargetCardInYourGraveyard) and `6665-6693` (T2, TargetCardInGraveyard) follow Path A from the plan:

- `combat_ref = state.combat.as_ref()` extracted as a `let` binding ABOVE the `.find(...)` call.
- Inside the closure, `role_ok`, `tapped_ok`, `untapped_ok` are computed as local lets at the top of the closure body.
- The terminal expression chains `obj.zone == controller_gy && matches_filter(...) && (!exclude_self || ...) && role_ok && tapped_ok && untapped_ok`.
- `filter` is captured by reference from the enclosing pattern match arm — no `.clone()` needed.
- `combat_ref` lifetime: borrows from `state.combat`, which outlives the closure since `.find()` runs synchronously on `state.objects.iter()`.

Build-clean confirmation: the merged commit passed `cargo build --workspace` (per the runner's gate claim and the fact that it was merged to main). The 4-way tuple match is exhaustive (all 4 (bool, bool) combinations enumerated); clippy does not warn.

## HASH Bump Completeness Audit (per dispatch brief item 4)

- `HASH_SCHEMA_VERSION` at `state/hash.rs:157` is `22`.
- History doc-comment at lines 146-156 adds the PB-XA2 entry citing all 3 new fields, CR rules, the 10 enforcement sites, the new `CombatState::is_blocking` helper, the `passes_combat_role` four-way match, and backward compatibility via `#[serde(default)] false`.
- All 16 pre-existing canary sentinel files in `crates/engine/tests/` bumped to `22u8` with PB-XA2-cited messages (grep `"PB-XA2 bumped HASH_SCHEMA_VERSION"` returns 16 + 2 in the new test file = 18 hits across 16 files).
- Grep for stale `HASH_SCHEMA_VERSION,? 21u8` across the workspace returns ZERO matches in `crates/`. (The only `21u8` references remaining are in `memory/primitives/pb-review-EAT.md`, which is the prior review file and is correctly frozen.)
- The 3 hash arms added at `hash.rs:4382-4387` correctly hash `is_blocking`, `is_tapped`, `is_untapped` (one per line, with CR-citation comments).

Bump completeness: VERIFIED.

## Discriminator-Test Discipline (per dispatch brief item 1 — highest-risk item)

**G-1 verification**:

| Check | Status | Evidence |
|-------|--------|----------|
| Sitter (non-blocker) added FIRST | YES | `tests/primitive_pb_xa2.rs:968` — `.object(sitter)` precedes `.object(defender)` |
| Defender (blocker) added SECOND | YES | `tests/primitive_pb_xa2.rs:970` — `.object(defender)` after sitter |
| `assert!(sitter_id < defender_id)` sanity guard | YES | `tests/primitive_pb_xa2.rs:982-989` — includes diagnostic message with both ObjectIds and reminder to adjust order if builder semantics change |
| Mental-toggle: removing `passes_combat_role` at T4 makes the test FAIL | VERIFIED by reasoning | Without enforcement, picker walks `state.objects` in ObjectId-ascending order. After SBA processing, Sitter (smaller ObjectId, on battlefield, is_creature) passes `matches_filter` + `ctrl_ok` (TargetController::Any default) + `passes_self` (source is `dying_scout`, distinct from Sitter). `.find()` short-circuits and returns Sitter. Assertion `target_id == defender_id` fails with `Got id ObjectId(<sitter_id>)`. WITH enforcement, Sitter fails `passes_combat_role` (not in `combat.blockers`); picker advances to Defender — assertion passes. Discriminator is genuine. |

**F-1/F-2/F-3 OR-semantics discriminators**: Each uses a single creature target and asserts pass/reject directly. No iteration-order ambiguity. F-3 specifically tests the rejection arm with a battlefield non-combatant (Peaceful Bear) — confirms that (T,T) requires membership in EITHER role, not just any battlefield presence.

**D-1/D-2 and E-1/E-2 are tap-state discriminators**: paired positive/negative tests; D-2/E-2 explicitly mutate `state.objects[id].status.tapped` before activation. Mutual discriminators (positive filter on negative state must err; positive filter on positive state must succeed).

**H-1 graveyard arm discriminator**: 3 sub-cases lock in:
- `is_blocking=true` on graveyard → REJECTS (graveyard never in `combat.blockers`).
- `is_tapped=true` on graveyard → REJECTS (default `status.tapped=false`).
- `is_untapped=true` on graveyard → ACCEPTS (degenerate-but-documented; design-quirk locked in).

G-1 conforms to H-XA-01's positive-discriminator requirement. No tautology. PASS.

## OOS-XA2 Seed Audit

Read `memory/primitives/pb-retriage-CC.md:1141-1196`. 5 seeds filed, all brief (1 paragraph each), with appropriate cross-references to PB-XA OOS entries and the plan:

| OOS ID | Topic | Audit |
|--------|-------|-------|
| OOS-XA2-1 | Target-side color predicate audit | Audit-only; routing already correct via `matches_filter`. LOW priority. References `cards/card_definition.rs:2526-2530`. |
| OOS-XA2-2 | Target-side `has_name` enforcement audit | Audit-only. LOW priority. References `matches_filter`. |
| OOS-XA2-3 | Target-side `is_nontoken` enforcement (carryforward from OOS-XA-3) | Correctly carries forward; PB-XA2 explicitly does not address. MEDIUM priority. References pre-existing OOS-XA-3. |
| OOS-XA2-4 | `CombatRole` enum refactor | Future refactor; documents the PB-XA reviewer recommendation that PB-XA2 chose not to take for scope reasons. LOW priority. References pb-review-XA + plan OR-semantics decision section. |
| OOS-XA2-5 | Runtime-predicate helper extraction (carryforward from E-XA-01) | Light refactor (~80 LOC net negative). LOW priority. References pb-review-XA E-XA-01 + plan Step 10 deferred question 2. |

All 5 OOS seeds correctly filed, no duplication of PB-XA content, appropriate cross-references. PASS.

## Doc-Comment Cross-Reference Audit (per dispatch brief item 7)

| Field | Cross-references to OR semantics? | Template adherence |
|-------|-----------------------------------|--------------------|
| `is_attacking` (`card_definition.rs:2600-2618`) | YES — lines 2605-2607 note OR semantics when `is_blocking` also set, refs `passes_combat_role` | "Enforced at:" template intact; full enumeration |
| `is_blocking` (`card_definition.rs:2619-2637`) | YES — lines 2624-2626 mirror the cross-reference back to `is_attacking` | Same template; full enumeration |
| `is_tapped` (`card_definition.rs:2638-2651`) | N/A (no OR semantics with `is_attacking`/`is_blocking`); calls out `is_untapped` unreachable-filter caveat | Same template; full enumeration |
| `is_untapped` (`card_definition.rs:2652-2659`) | N/A; brief reference to `is_tapped` | Shortened "same sites as `is_tapped`" — see E-XA2-01 |

Cross-references are consistent and bidirectional for the `is_attacking`/`is_blocking` pair. The OR-semantics decision is properly documented at the source-code level in both directions.

## Previous Findings

This is the initial independent review for PB-XA2 (no prior independent review existed). PB-XA's H-XA-01 lesson was correctly applied at implementation time:

| Lesson | Application | Verified |
|--------|-------------|----------|
| H-XA-01 (PB-XA): F-1 positive discriminator must use ObjectId-ordering with non-attacker FIRST | G-1 in PB-XA2 explicitly puts Sitter FIRST + sanity guard + mental-toggle reasoning | YES — see Discriminator-Test Discipline above |
| E-XA-01 (PB-XA): runtime-predicate duplication | Carried forward as OOS-XA2-5 | YES — properly deferred |
| L-XA-02 (PB-XA): test sentinel naming `test_pb_hash_schema_version_live_sentinel` | A-1 in PB-XA2 uses the renamed convention | YES — line 107 |

---

## Tooling Verification (per dispatch brief)

The reviewer agent does not have shell access. The runner's gate claims (per merge commit `9eca57af`):
- `cargo build --workspace`: clean.
- `cargo test --workspace`: green (~2805 tests expected per plan, runner claimed pass).
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo fmt --check`: clean.

Spot-check of diff confirms:
- No `.unwrap()` introduced in library code (test code uses `.unwrap_or_else(|| panic!(...))` and `.expect(...)`, which is acceptable per conventions).
- Match exhaustiveness: 4-way tuple match on `(bool, bool)` is exhaustive.
- No new `StackObjectKind`/`KeywordAbility`/`Effect` variants — no TUI/replay-viewer match-arm risk.
- Hash arms order is preserved (PB-XS `exclude_self` precedes PB-XA2 fields in `hash.rs:4380-4387`, matching the order in `card_definition.rs:2683-2659` field declarations).

If the coordinator wants full confirmation, the merged commit on main should pass:
```
cd /home/skydude/projects/scutemob
cargo test --workspace 2>&1 | tail -20
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -20
cargo fmt --check
```

---

## Summary

PB-XA2 is a textbook mechanical primitive extension. The 3-predicate template was applied uniformly across all 10 enforcement sites with symmetric placement and consistent CR citations. The OR-semantics decision for "attacking or blocking creature" was implemented cleanly as a single `passes_combat_role` four-way tuple match — minimal scope, maximum clarity. The G-1 discriminator test, the highest-risk item per project history (PB-XA's H-XA-01 was a tautology that required a fix-phase), was implemented correctly on the first try with explicit object-add ordering, a `sitter_id < defender_id` sanity guard, and documented mental-toggle reasoning. The HASH 21→22 bump propagated cleanly across all 16 canary sentinels with uniform messages. Eiganjo, Seat of the Empire's Channel half is now correctly enforced with both `is_attacking` and `is_blocking` set, matching its oracle text.

The 3 LOW findings are stylistic only: `is_untapped` doc terseness (E-XA2-01), abilities.rs single-line vs casting.rs multi-line formatting (E-XA2-02), and the documented `is_untapped=true` graveyard-arm degenerate edge case (E-XA2-03, locked in by H-1c). None block ship. None warrant a follow-up fix commit.

**VERDICT: PASS** (0 HIGH, 0 MEDIUM, 3 LOW; no NEEDS-FOLLOWUP). The merged main is correct as-shipped. LOWs may be addressed opportunistically (likely natural cleanup targets when OOS-XA2-5 helper extraction lands).
