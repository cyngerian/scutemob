# Primitive Batch Plan: PB-XA — TargetFilter.is_attacking enforcement at validate sites

**Generated**: 2026-05-15
**Worker**: scutemob-24 (`feat/pb-xa-targetfilterisattacking-enforcement-at-validate-sites`)
**Predecessor**: PB-XS (`scutemob-21`, shipped 2026-05-14) added `TargetFilter.exclude_self`
and the per-call-site validator pattern. PB-XA reuses the same pattern for a sibling
runtime-relationship field that already exists (`TargetFilter.is_attacking`) but is
silently ignored by `validate_object_satisfies_requirement` and the trigger
auto-target picker.
**CR Rules**: CR 109.1 (object identity), CR 508.1k ("attacking creature"),
CR 601.2c (target-selection legality), CR 603.3d (trigger no-legal-target skip).
**Cards in scope**: **1** — Thousand-Faced Shadow
(`crates/engine/src/cards/defs/thousand_faced_shadow.rs:47-52` — ETB trigger:
"create a token that's a copy of another target attacking creature"). Its current
def already has `is_attacking: true` and `exclude_self: true` set on the filter,
but only `exclude_self` is enforced — the def carries a TODO callout that PB-XA
removes once the engine bites.

**HASH bump**: NONE. `is_attacking` is a pre-existing TargetFilter field hashed at
`hash.rs:4347`. The existing live `HASH_SCHEMA_VERSION = 20u8` sentinel must
remain unchanged. AC 3.

---

## Executive Summary

PB-XA is a **~12-line mechanical enforcement extension** to mirror PB-XS exactly.
After `matches_filter` returns true at each call site, add a runtime check
`!filter.is_attacking || combat_attacking(state, candidate_id)`. The PB-XS pattern
already threads `self_id: Option<ObjectId>` through `validate_targets_with_source`
and reaches the four filter-bearing TargetRequirement variants; we extend each of
those four arms PLUS the six auto-target picker sites in `abilities.rs`. No
struct changes, no discriminants, no schema/hash impact.

| # | Site | File:Line | Variant | Existing PB-XS Check |
|---|------|-----------|---------|----------------------|
| V1 | validate_object_satisfies_requirement, TargetCreatureWithFilter | casting.rs:5707-5726 | declarative | exclude_self at 5723 |
| V2 | validate_object_satisfies_requirement, TargetPermanentWithFilter | casting.rs:5727-5746 | declarative | exclude_self at 5743 |
| V3 | validate_object_satisfies_requirement, TargetCardInYourGraveyard | casting.rs:5749-5756 | declarative graveyard | exclude_self at 5754 |
| V4 | validate_object_satisfies_requirement, TargetCardInGraveyard | casting.rs:5758-5763 | declarative graveyard | exclude_self at 5761 |
| T1 | trigger picker, TargetCardInYourGraveyard | abilities.rs:6625-6645 | graveyard scan | exclude_self at 6639 |
| T2 | trigger picker, TargetCardInGraveyard | abilities.rs:6646-6658 | graveyard scan | exclude_self at 6653 |
| T3 | trigger picker, TargetCreatureWithFilter (top-level) | abilities.rs:6744-6770 | battlefield scan | exclude_self at 6768 |
| T4 | trigger picker, TargetPermanentWithFilter (top-level) | abilities.rs:6771-6794 | battlefield scan | exclude_self at 6792 |
| T5 | trigger picker, TargetCreatureWithFilter (UpToN inner) | abilities.rs:6835-6848 | battlefield scan | exclude_self at 6845 |
| T6 | trigger picker, TargetPermanentWithFilter (UpToN inner) | abilities.rs:6849-6860 | battlefield scan | exclude_self at 6858 |

For V3/V4/T1/T2 (graveyard arms), `state.combat.attackers.contains_key(&id)` will
always return `false` for graveyard objects (CR 508.1k requires battlefield), so
the check correctly rejects any candidate when `is_attacking` is set. We apply
the check uniformly across all four filter variants for consistency with PB-XS
(symmetric "this runtime predicate gates all variants").

---

## Full-Chain Verification

### Thousand-Faced Shadow — Ninjutsu `{2}{U}{U}` — In scope

**Oracle**: "When this creature enters from your hand, if it's attacking, create
a token that's a copy of **another target attacking creature**. The token enters
tapped and attacking."

| Hop | Status | Notes |
|-----|--------|-------|
| ETB trigger | OK | `WhenEntersBattlefield` already supported. |
| TargetFilter.exclude_self | OK | Already enforced by PB-XS. |
| TargetFilter.is_attacking | **BITING AFTER PB-XA** | Currently ignored at validate sites; PB-XA gates target selection on `combat.attackers.contains_key(id)`. |
| Effect::CreateTokenCopy | OK | Pre-existing. |
| Dispatch chain | OK | Trigger queueing → ETB trigger auto-target picker (T3 + T4 paths) → resolution. The picker site that bites is the top-level `TargetPermanentWithFilter` arm (T4 / abilities.rs:6771-6794) because the def uses `TargetPermanentWithFilter` (creature constrained via `has_card_type`). |

**Authorability after PB-XA**: YES, with the inline TODO at
`thousand_faced_shadow.rs:44-46` deleted as part of this PB.

### aetherize / blessed_alliance — OUT OF SCOPE

Both cards use `is_attacking: true`, but **on `Effect::BounceAll.filter` and
`Effect::SacrificePermanents.filter`** — not on `TargetRequirement` filters.
Those are resolution-time `effects/mod.rs:matches_filter` consumers already
gating on `is_attacking` (see `effects/mod.rs:2686-2698`). PB-XA does not touch
the effect-resolution path. Confirmed via grep
(`crates/engine/src/cards/defs/{aetherize.rs,blessed_alliance.rs}`).

### Eiganjo, Seat of the Empire — Authorability NOT improved by PB-XA

Eiganjo's Channel ability targets "attacking or blocking creature." Current def
uses bare `TargetCreature` and carries a TODO citing **two** missing fields:
`is_attacking` (now enforceable after PB-XA) AND `is_blocking` (no field exists
on TargetFilter). PB-XA does NOT add `is_blocking`; the card def remains a TODO
gap. The `is_blocking` half is an OOS-XA seed (see AC 6).

---

## Step-By-Step

### Step 1 — Engine enforcement at validate sites (casting.rs)

For each of V1..V4, after the existing `passes_filter` / `passes_controller` /
`passes_self` (exclude_self) computation, add a new boolean
`passes_attacking` AND it into the final result. The runtime predicate is:

```rust
let passes_attacking = !filter.is_attacking
    || state
        .combat
        .as_ref()
        .is_some_and(|c| c.attackers.contains_key(&id));
```

Pattern matches the existing `effects/mod.rs:2689-2698` shape. The check goes
INSIDE the if-let branch that already early-returns false for wrong-zone /
wrong-type cases (e.g., V1's `if !on_battlefield || !is_creature`), so the
combat lookup runs only after the cheap zone/type guards.

Insert site for each variant (PB-XS-style comment-then-let-then-AND):

```rust
// PB-XA: CR 508.1k / 601.2c — "target attacking X" restricts target
// selection to creatures currently in combat.attackers. Like exclude_self,
// is_attacking is a runtime relationship NOT checked by matches_filter.
let passes_attacking = !filter.is_attacking
    || state.combat.as_ref().is_some_and(|c| c.attackers.contains_key(&id));
passes_filter && passes_controller && passes_self && passes_attacking
```

For graveyard arms V3/V4 the final AND chain becomes:
`in_your_gy && matches_filter && passes_self && passes_attacking` (or
`in_any_gy && ... && passes_attacking`).

### Step 2 — Engine enforcement at trigger auto-target picker (abilities.rs)

For each of T1..T6, mirror the same pattern. The picker's closures already have
`state` in scope. Use `obj.id` for the candidate ObjectId. The local variable
name `passes_attacking` keeps the diff symmetric with the `passes_self` line
already present. Final closure result becomes:
`passes && ctrl_ok && passes_self && passes_attacking`.

For T1/T2 (graveyard scan), insert the check inline in the `.find` closure
predicate next to the existing `(!filter.exclude_self || obj.id != trigger.source)`
clause.

### Step 3 — Card def cleanup

`crates/engine/src/cards/defs/thousand_faced_shadow.rs`: delete the 3-line TODO
comment at lines 44-46 (the "still silently ignored" callout) and replace with a
short note that PB-XA enforces both flags. Oracle text and `is_attacking: true`
stay.

### Step 4 — Update doc comment on TargetFilter.is_attacking

`crates/engine/src/cards/card_definition.rs:2600-2605` — the doc comment claims
"NOT checked inside `matches_filter()`. It MUST be checked explicitly at each
call site that uses it." Add an "Enforced at:" block mirroring `exclude_self`
(2640-2648) listing the validate + auto-target-picker enforcement sites.

### Step 5 — Tests in `tests/primitive_pb_xa.rs`

Mirror `primitive_pb_xs.rs` structure (sections A through G), substituting
`is_attacking` semantics for `exclude_self`. Minimum coverage:

| ID | Section | What | Why |
|----|---------|------|-----|
| A-1 | hash sentinel | `assert_eq!(HASH_SCHEMA_VERSION, 20u8)` | AC 3: no bump expected |
| B-1 | PartialEq | filters differing only on `is_attacking` are not equal | invariant — `is_attacking` is hashed (pre-existing) |
| C-1 | activated target, declarative path, **negative** | activated ability with TargetCreatureWithFilter{is_attacking=true} rejects non-attacking creature | exercises V1 |
| C-2 | activated target, declarative path, **positive** | same ability accepts a creature in `combat.attackers` | exercises V1 happy path |
| D-1 | activated target, TargetPermanentWithFilter, **negative** | rejects non-attacking permanent | exercises V2 |
| D-2 | TargetPermanentWithFilter, **positive** | accepts attacking permanent | exercises V2 happy path |
| E-1 | graveyard arm V3/V4, no-combat-on-graveyard | TargetCardInYourGraveyard{is_attacking=true} rejects an Elf card in graveyard even when combat is active | exercises V3 — graveyard objects are never in `combat.attackers` |
| F-1 | trigger auto-target picker, **positive** (Thousand-Faced-Shadow-shaped ETB) | ETB trigger auto-picks an attacking creature over a non-attacking one | exercises T4 (top-level TargetPermanentWithFilter) |
| F-2 | trigger auto-target picker, **negative** (no attacker present) | trigger SKIPPED when no creature in combat satisfies filter, per CR 603.3d | exercises T4 no-legal-target skip |
| G-1 | matches_filter unit | `matches_filter` ignores `is_attacking` (existing invariant, regression sentinel) | mirrors PB-XS G-1 |

C-2 / D-2 / F-1 require initialising `CombatState.attackers` with the candidate
ObjectId mapped to an `AttackTarget::Player(p(2))`. Test harness builder pattern
already supports `with_combat` / direct `state.combat = Some(...)` — pattern
matches existing combat tests.

### Step 6 — Build / clippy / fmt / review (AC 7)

Standard sequence:
1. `cargo build --workspace` — clean.
2. `cargo test --workspace` — green, expect `2764+N` tests (N = number of new
   PB-XA tests, ~10).
3. `cargo clippy --workspace --all-targets -- -D warnings` — clean.
4. `cargo fmt --check` — clean.
5. Spawn `primitive-impl-reviewer` agent against the PB-XA diff. Resolve
   HIGH/MEDIUM findings inline.

### Step 7 — File OOS-XA seeds (AC 6)

Append to `memory/primitives/pb-retriage-CC.md`:

- **OOS-XA-1**: `TargetFilter.is_blocking` — Eiganjo, Seat of the Empire Channel
  ability ("target attacking or blocking creature"). Requires a new
  `is_blocking: bool` TargetFilter field; enforcement parallel to is_attacking
  via `combat.is_blocking(id)` (need new CombatState helper checking
  `attackers.values().any(|t| matches!(t, AttackTarget::Object(_)))` interaction
  — actually `blockers` map). Yield: 1+ cards (Eiganjo confirmed; Reckless
  Charge family possible). Light primitive bundle.
- **OOS-XA-2**: `TargetFilter.is_tapped` / `is_untapped` — many cards target
  "tapped creature" (Animate Dead-style reanimation, Murderous Cut). Pre-existing
  candidate; field doesn't exist on TargetFilter. Possible PB-XA-2 sibling.
- **OOS-XA-3**: target-side `is_nontoken` — TargetFilter already has
  `is_nontoken` field on the receiver side; verify validate-time enforcement is
  parallel to `is_attacking` (grep showed effects/mod.rs at line 2683 enforces
  it for effect-resolution, but unclear if target-validation does). Defer
  re-audit to next runtime-predicate PB.

Worker emits these as OOS-XA-{1,2,3} blocks at the bottom of `pb-retriage-CC.md`
under a "## OOS seeds filed by PB-XA (scutemob-24, 2026-05-15)" heading.

---

## Files Modified (expected)

- `crates/engine/src/rules/casting.rs` — 4 sites (V1-V4), ~24 lines added.
- `crates/engine/src/rules/abilities.rs` — 6 sites (T1-T6), ~36 lines added.
- `crates/engine/src/cards/card_definition.rs` — doc comment update on
  `TargetFilter.is_attacking`.
- `crates/engine/src/cards/defs/thousand_faced_shadow.rs` — remove TODO callout.
- `crates/engine/tests/primitive_pb_xa.rs` — NEW, ~700 LOC test file mirroring
  `primitive_pb_xs.rs`.
- `memory/primitives/pb-retriage-CC.md` — append OOS-XA-{1,2,3}.

No changes expected to:
- `state/hash.rs` (HASH stable at 20)
- `cards/card_definition.rs` (no new field)
- `replay_harness.rs` (no schema flow)

---

## Acceptance Map

| AC | Step(s) | Verifiable by |
|----|---------|---------------|
| 3847 (validate enforcement) | Step 1, V1-V4 | cargo test C/D/E sections |
| 3848 (trigger picker enforcement) | Step 2, T1-T6 | cargo test F section |
| 3849 (no HASH bump) | Steps 1-6 untouched hash.rs | Step 5 A-1 sentinel test green |
| 3850 (cards) | Step 3 + sweep verification (done in plan) | grep `is_attacking: true` in defs |
| 3851 (tests) | Step 5 | cargo test, +10 tests |
| 3852 (OOS seeds) | Step 7 | pb-retriage-CC.md diff |
| 3853 (build/clippy/fmt/review) | Step 6 | tooling output |

---

## Open Questions

None — this is a mechanical extension of PB-XS. The plan-phase verification
confirms scope is 1 card with the def already authored. No re-triage needed.
