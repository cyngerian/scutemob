# Primitive Batch Plan: PB-TS — TokenSpec.count u32 → EffectAmount (dynamic token-count primitive)

**Generated**: 2026-04-30
**Primitive**: Make `TokenSpec.count` accept a dynamic `EffectAmount` instead of being a fixed
`u32`. `Effect::CreateToken` and `Effect::CreateTokenAndAttachSource` resolve the count via the
existing `resolve_amount(state, amount, ctx) -> i32` helper at execution time, then feed the
resolved `u32` through the unchanged `apply_token_creation_replacement` doubling boundary.
**CR Rules**: 111.1, 111.4, 614.1, 614.1c, 113.7 / 113.7a, 608.2h, 122.1, 122.6
**Cards affected**: 4 (Phyrexian Swarmlord, Chasm Skulker, Krenko Mob Boss, Izoni Thousand-Eyed —
all currently TODO'd at the token-count line)
**Dependencies**: PB-CC-A `EffectAmount::PlayerCounterCount` (DONE); existing `EffectAmount`
variants `CounterCount`, `PermanentCount`, `CardCount` (DONE pre-PB)
**Deferred items from prior PBs**: Phyrexian Swarmlord N/A-deferred from PB-CC-A pending PB-TS;
Chasm Skulker / Krenko / Izoni TODO'd indefinitely until PB-TS

---

## Step 1 — CR research (verbatim from MCP rules server)

### CR 111.1 / 111.4 — token creation

> 111.1. Some effects put tokens onto the battlefield. A token is a marker used to represent any
> permanent that isn't represented by a card.
>
> 111.4. A spell or ability that creates a token sets both its name and its subtype(s). If the
> spell or ability doesn't specify the name of the token, its name is the same as its subtype(s)
> plus the word "Token." Once a token is on the battlefield, changing its name doesn't change
> its subtype(s), and vice versa.

### CR 614.1 / 614.1c — replacement effects, including ETB-shape replacements

> 614.1. Some continuous effects are replacement effects. […] Such effects watch for a particular
> event that would happen and completely or partially replace that event with a different event.
>
> 614.1c. Effects that read "[This permanent] enters with …," "As [this permanent] enters …," or
> "[This permanent] enters as …" are replacement effects.

Implication: token-doubling replacements (Doubling Season, Anointed Procession, Parallel Lives,
Mondrak) match `WouldCreateTokens` and modify the **resolved count**, not the
`EffectAmount` expression. The replacement boundary in `apply_token_creation_replacement`
takes a `u32` count and returns a `u32` count — that signature stays as-is. Resolution of the
dynamic count occurs **before** the replacement layer.

### CR 113.7 / 113.7a — source identity, LKI, and queries deferred to put-on-stack vs. resolve

> 113.7. The source of an ability is the object that generated it. […]
>
> 113.7a. Once activated or triggered, an ability exists on the stack independently of its source.
> Destruction or removal of the source after that time won't affect the ability. […] any
> activated or triggered ability that references information about the source for use while
> announcing an activated ability or putting a triggered ability on the stack checks that
> information when the ability is put onto the stack. Otherwise, it will check that information
> when it resolves. **In both instances, if the source is no longer in the zone it's expected to
> be in at that time, its last known information is used.**

Load-bearing for Chasm Skulker: the death trigger ("when Chasm Skulker dies, create X 1/1
Squids where X = +1/+1 counters on it") fires from the LKI; `ctx.source` is the dead object's
ID, and its counters persist through `move_object_to_zone` (Toothy precedent at
`crates/engine/src/cards/defs/toothy_imaginary_friend.rs:43-58` proves
`EffectAmount::CounterCount { target: Source, counter: PlusOnePlusOne }` resolves correctly
post-death).

### CR 608.2h — spell-effect "answer determined only once, when the effect is applied"

> 608.2h. If an effect requires information from the game (such as the number of creatures on
> the battlefield), the answer is determined only once, when the effect is applied. If the
> effect requires information from a specific object, including the source of the ability
> itself, the effect uses the current information of that object if it's in the public zone it
> was expected to be in; if it's no longer in that zone, or if the effect has moved it from a
> public zone to a hidden zone, the effect uses the object's last known information. See rule
> 113.7a. […]

Token count IS an "answer determined only once, when the effect is applied." The dynamic count
is resolved at `Effect::CreateToken` execution time (not at trigger-fire time, not at cast
time, not at ETB replacement time) and stays fixed for the duration of that effect. This is the
correct semantic for all four target cards: Krenko taps, the count is locked at activated-
ability resolution; Phyrexian Swarmlord upkeep trigger resolves, the count is locked at trigger
resolution; Chasm Skulker death trigger resolves, the count is locked at death-trigger
resolution (LKI); Izoni ETB trigger resolves, the count is locked at ETB resolution.

### CR 122.1 + 122.6 — counter timing (LKI invariant)

> 122.1. A counter is a marker placed on an object or player that modifies its characteristics
> and/or interacts with a rule, ability, or effect. […]
>
> 122.6. Some spells and abilities refer to counters being put on an object. This refers to
> putting counters on that object while it's on the battlefield and also to an object that's
> given counters as it enters the battlefield.

Counters on the source persist into LKI on zone change. Verified in `move_object_to_zone`
(GameObject moves with `counters` map intact prior to characteristics being reset);
Toothy/Camellia precedents both rely on this.

---

## Step 2 — Engine architecture walk (full dispatch chain)

### Site 1 — DSL: `TokenSpec.count` field at `crates/engine/src/cards/card_definition.rs:3099-3166`

Today: `pub count: u32,` at line 3114; `Default` sets `count: 1` at line 3156.

Required change (Shape A — see Step 3): replace `count: u32` with `count: EffectAmount`.
`Default` becomes `count: EffectAmount::Fixed(1)`. The serde format changes (a u32 wire
becomes a tagged enum), so the hash version bump (Site 7) is mandatory.

### Site 2 — Predefined helpers at `crates/engine/src/cards/card_definition.rs:3167-3388`

Six helpers take `count: u32` parameters and assign to the field directly:
`treasure_token_spec(count: u32)` (3171), `food_token_spec(count: u32)` (3194),
`clue_token_spec(count: u32)` (3243), `blood_token_spec(count: u32)` (3295),
`zombie_decayed_token_spec(count: u32)` (3370). `army_token_spec(subtype: &str)` (3344) hard-
codes `count: 1`.

Required change: keep the `count: u32` API (caller ergonomics — most callers pass a literal
1), but internally store `count: EffectAmount::Fixed(count as i32)`. Helpers that hard-code
1 store `count: EffectAmount::Fixed(1)`. **Do NOT force callers to wrap `EffectAmount::Fixed(N)`
for static counts.**

### Site 3 — `Effect::CreateToken` at `crates/engine/src/effects/mod.rs:540-585`

Today (line 546): `apply_token_creation_replacement(state, ctx.controller, spec.count)`.

Required change:
```rust
let raw = resolve_amount(state, &spec.count, ctx);
let resolved = raw.max(0) as u32;
let (token_count, repl_events) =
    crate::rules::replacement::apply_token_creation_replacement(state, ctx.controller, resolved);
```
The `for _ in 0..token_count` loop at line 561 already takes a u32 — no further change. The
`enters_attacking` check (553-560) is independent.

CR cite: 608.2h ("answer determined only once, when the effect is applied") — the resolution
order is **resolve dynamic count → apply replacement (Doubling Season etc.) → loop**.

### Site 4 — `Effect::CreateTokenAndAttachSource` at `crates/engine/src/effects/mod.rs:595-660`

Today (line 597): `for _ in 0..spec.count { … }` (no replacement-effect application — Living
Weapon is the only user, and the planner must verify whether token doublers also apply here).

Required change:
```rust
let raw = resolve_amount(state, &spec.count, ctx);
let resolved = raw.max(0) as u32;
let (token_count, repl_events) =
    crate::rules::replacement::apply_token_creation_replacement(state, ctx.controller, resolved);
events.extend(repl_events);
let mut first_token_id: Option<ObjectId> = None;
for _ in 0..token_count { … }
```

**Audit note**: today's code does NOT call `apply_token_creation_replacement` for Living Weapon.
This is technically a pre-existing bug (a Doubling Season + Living Weapon should produce two
Germ tokens). The runner SHOULD restore the call here as part of the migration since the
dispatch site now needs `resolve_amount` anyway. **Stop-and-flag if uncertain**: a pure
mechanical migration could leave the missing-replacement bug intact and add the live-eval call
only. Recommend running existing Living Weapon + Doubling Season tests first to determine the
current expected behavior; if there are no such tests, leave the replacement logic absent and
flag it as a separate seed.

### Site 5 — `apply_token_creation_replacement` at `crates/engine/src/rules/replacement.rs:2603-2637`

Signature: `pub fn apply_token_creation_replacement(state, controller, count: u32) -> (u32, Vec<GameEvent>)`.

Required change: **NONE**. The u32 boundary is intentional. Doubling math (`modified_count *= 2`
at line 2628) operates on the resolved count. `EffectAmount` resolution happens upstream of
this entrypoint.

### Site 6 — Internal helpers using `spec.count`: `crates/engine/src/state/dungeon.rs:88-185, 372`

`dungeon.rs` defines five private TokenSpec helpers (`goblin_token_spec` line 88,
`treasure_token_spec_1` line 107, `skeleton_11_token_spec` line 126, `skeleton_41_menace_token_spec`
line 145, `atropal_token_spec` line 164) — all use literal `count: 1`. Plus the inline
`spec.count = 2` at line 372 (Muiral's Graveyard — "Create two 1/1 black Skeleton creature
tokens").

Required change:
- Each `count: 1` → `count: EffectAmount::Fixed(1)`.
- `spec.count = 2` (line 372) → `spec.count = EffectAmount::Fixed(2)`.

### Site 7 — `Effect::Investigate` at `crates/engine/src/effects/mod.rs:661-686`

Already constructs `clue_token_spec(1)` (line 664). After the helper change, no further
modification needed.

### Site 8 — Test fixture at `crates/engine/tests/tapped_and_attacking.rs:29-42`

Defines `human_token_spec_attacking` with literal `count: 2` (line 37).

Required change: `count: EffectAmount::Fixed(2)`. Add `EffectAmount` to the imports.

### Site 9 — Sentinel assertion at `crates/engine/tests/blood_tokens.rs:812, 826`

```rust
assert_eq!(spec.count, 1, "blood_token_spec(1) creates 1 token");   // line 812
assert_eq!(spec3.count, 3, "blood_token_spec(3) creates 3 tokens"); // line 826
```

Required change: assertions become `assert_eq!(spec.count, EffectAmount::Fixed(1), …)` and
`assert_eq!(spec3.count, EffectAmount::Fixed(3), …)`. `EffectAmount` derives `PartialEq`
(verified at `card_definition.rs:2246`), so equality assertion works directly.

### Site 10 — Hash impl at `crates/engine/src/state/hash.rs:4290-4309`

`HashInto for TokenSpec` calls `self.count.hash_into(hasher)` at line 4300. `EffectAmount`
already implements `HashInto` (used by `EffectAmount::PlayerCounterCount` etc. in PB-CC-A).

Required change: **NONE to the call** (the trait dispatch picks up `EffectAmount` automatically
once the field type changes). However:
- `HASH_SCHEMA_VERSION` MUST bump 13 → 14 (Site 11).
- History entry 14 must be appended (Site 11).

### Site 11 — `HASH_SCHEMA_VERSION` at `crates/engine/src/state/hash.rs:67`

Today: `pub const HASH_SCHEMA_VERSION: u8 = 13;` (PB-CC-C-followup).

Required change: bump to `14`. Append history entry 14 after line 66:
```
/// - 14: PB-TS (2026-04-30) — `TokenSpec.count` shape change u32 → EffectAmount;
///   `Effect::CreateToken` and `Effect::CreateTokenAndAttachSource` resolve dynamic
///   counts via `resolve_amount` at execution time (CR 608.2h, CR 111.1).
///   Replacement boundary `apply_token_creation_replacement` keeps u32 signature
///   (token doublers operate on resolved counts). Wire format of TokenSpec.count
///   changes from raw u32 to tagged-enum EffectAmount; pre-PB-TS replays are not
///   forward-compatible. Unblocks Phyrexian Swarmlord, Chasm Skulker, Krenko Mob Boss,
///   Izoni Thousand-Eyed.
```

### Site 12 — Sentinel-assertion test files (sweep targets)

Verified via rust-analyzer workspace_symbols (`hash_schema_version` query, results dated
2026-04-30):

| File | Function | Line | Notes |
|---|---|---|---|
| `crates/engine/tests/primitive_pb_cc_a.rs` | `test_hash_schema_version_after_pb_cc_c_followup` | 99 | Update assertion `13` → `14`; rename to `…_after_pb_ts`; update message to cite PB-TS / TokenSpec shape change. |
| `crates/engine/tests/primitive_pb_cc_c_followup.rs` | `test_hash_schema_version_after_pb_cc_c_followup` | 394 | Update assertion `13u8` → `14u8`; same rename + message update. |
| `crates/engine/tests/pbt_up_to_n_targets.rs` | `test_pbt_hash_schema_version_is_13` | 404 | Rename to `…_is_14`; update message. |
| `crates/engine/tests/pbt_up_to_n_targets.rs` | `test_pbt_hash_schema_version_sentinel_is_13_regression` | 864 | Rename to `…_is_14_regression`; update message. |
| `crates/engine/tests/effect_sacrifice_permanents_filter.rs` | `test_sft_hash_schema_version_is_13` | 134 | Rename to `…_is_14`; update message. |

The runner MUST rerun `rust_analyzer_workspace_symbols(query: "hash_schema_version")` (or grep
for `13u8` and `HASH_SCHEMA_VERSION`) after the bump to verify no additional sentinel was
added between plan and impl. **5 files; 5 sentinels**.

### Site 13 — Replay harness at `crates/engine/src/testing/replay_harness.rs`

The harness round-trips `GameState`. Adding/changing a field on `TokenSpec` does NOT add a new
match arm — `TokenSpec` is a struct, not an enum. Verify by `cargo build --workspace` after
impl. **Expected**: clean build because the harness does not destructure TokenSpec.count.

### Site 14 — replay-viewer view_model + TUI stack_view (per `MEMORY.md` 50%-miss-rate gotcha)

`tools/replay-viewer/src/view_model.rs` and `tools/tui/src/play/panels/stack_view.rs` exhaustive-
match on `StackObjectKind` and `KeywordAbility`. Changing a struct field (TokenSpec.count) does
not add new variants. **Expected: no change required**, but the runner MUST run
`cargo build --workspace` (not just `cargo build`) after impl to surface any indirect breakage.

### Site 15 — All other card defs constructing `TokenSpec { …, count: N, … }` literals

Verified via rust-analyzer workspace symbols + grep for `count: 1` / `count: 2` / `count: 3` in
TokenSpec literals: zero hits in `crates/engine/src/cards/defs/` (card defs uniformly use
predefined helpers like `clue_token_spec(1)` or `treasure_token_spec(2)` — they do NOT
hand-construct `TokenSpec`). The only TokenSpec literal call sites outside the helpers
themselves are the dungeon.rs and tapped_and_attacking.rs sites enumerated above.

**Total migration site count**: 6 helper bodies + 5 dungeon helpers + 1 dungeon mutation + 1
test fixture + 4 card def re-authors (the unblocked targets) = **17 explicit edit sites**.
Default value in TokenSpec impl block adds 1. Hash version + history entry adds 1. Sentinel
sweep adds 5. **Engine-side migration cost: ~24 files touched**.

---

## Step 3 — Shape decision

**Chosen: Shape A — replace `count: u32` with `count: EffectAmount` directly.**

### Why Shape A

1. **Type-system enforcement** (per `feedback_verify_full_chain.md`): a single source of truth
   for token count. No runtime branching on "which field do I read?" — every consumer reads
   `spec.count` and pipes it through `resolve_amount`.
2. **Mirrors the precedent set by every other count field in the engine**: `Effect::DrawCards.count`,
   `Effect::GainLife.amount`, `Effect::Scry.count`, `Effect::AddCounter.count` (… already
   discussed; this last one is u32, but it post-dates the `EffectAmount` design and is itself
   a candidate for migration). The `EffectAmount::Fixed(N)` ergonomic cost is borne uniformly.
3. **Helper-API ergonomics preserved**: `treasure_token_spec(count: u32)` keeps its u32
   parameter; conversion to `EffectAmount::Fixed(count as i32)` happens internally. Callers
   already write `treasure_token_spec(2)`; nothing changes.
4. **Hash discipline is cleanest**: `EffectAmount`'s `HashInto` impl is already exhaustive over
   all variants (PB-CC-A added discriminant 16; the canonical hash already covers `Fixed(1)` ≠
   `Fixed(2)` ≠ `CounterCount{…}`, etc.). The TokenSpec hash arm needs no new logic —
   `self.count.hash_into(hasher)` dispatches through.
5. **Matches the precedent of PB-CC-C-followup choosing Shape A+D**: introducing a new variant
   in lockstep with a layered change to the dispatch site, rather than an overlay flag.

### Why NOT Shape B (`dynamic_count: Option<EffectAmount>` overlay)

- Two-source-of-truth footgun. Authors must remember to either set `count` OR `dynamic_count` —
  exactly the kind of bug PB-CC-C explicitly added doc-comments to prevent.
- All 17 existing dispatch sites would need to read `spec.dynamic_count.as_ref().unwrap_or(&fixed)`
  or similar, not just one. The runtime cost is also a soft footgun (silently fall through to
  `count` if the overlay is forgotten).
- Hash-arm discipline weaker: now both `count` and `dynamic_count` contribute to the hash, and
  `Some(Fixed(1))` and `count = 1, dynamic_count = None` are different bytes for the same
  semantic state. Either hash both fields and accept duplicates, or canonicalize at write
  time — both are fragile.

### Why NOT Shape C (`TokenCount::{Fixed(u32), Dynamic(EffectAmount)}` enum)

- Adds a third intermediate type with no expressive gain over `EffectAmount` directly.
  `EffectAmount::Fixed(i32)` already encodes "fixed count," so `TokenCount::Fixed(u32)` is
  redundant.
- Three layers of enum (`TokenCount::Dynamic(EffectAmount::CounterCount {…})`) adds matching
  boilerplate across all 17 dispatch sites for no semantic improvement.
- `Default::default()` becomes ambiguous (`Fixed(1)` vs `Dynamic(Fixed(1))`).

### What every dispatch site sees under Shape A

| Site | Before | After |
|---|---|---|
| Field decl | `pub count: u32,` | `pub count: EffectAmount,` |
| Default | `count: 1,` | `count: EffectAmount::Fixed(1),` |
| Helper construction | `count,` (u32 param) | `count: EffectAmount::Fixed(count as i32),` |
| Helper hardcoded | `count: 1,` | `count: EffectAmount::Fixed(1),` |
| `Effect::CreateToken` | `apply_token_creation_replacement(…, spec.count)` | `let resolved = resolve_amount(state, &spec.count, ctx).max(0) as u32; apply_token_creation_replacement(…, resolved)` |
| `Effect::CreateTokenAndAttachSource` | `for _ in 0..spec.count` | `let resolved = resolve_amount(state, &spec.count, ctx).max(0) as u32; for _ in 0..resolved` |
| `apply_token_creation_replacement` | `count: u32` parameter | unchanged (boundary preserved) |
| Test sentinel `assert_eq!(spec.count, 1, …)` | u32 literal compare | `EffectAmount::Fixed(1)` compare |
| Hash arm | `self.count.hash_into(hasher);` | unchanged (trait dispatch) |
| Hash version | 13 | 14 |

---

## Step 4 — Dispatch unification verdict

**PASS — single primitive batch, exactly 4 cards in scope.**

| Card | Change | EffectAmount expression |
|---|---|---|
| `phyrexian_swarmlord.rs` | Add upkeep `Triggered { trigger_condition: AtBeginningOfYourUpkeep, effect: CreateToken { spec: insect_token_spec_with_count(EffectAmount::PlayerCounterCount { player: EachOpponent, counter: Poison }) }}`. Token spec is hand-constructed (1/1 green Phyrexian Insect with Infect). | `EffectAmount::PlayerCounterCount { player: PlayerTarget::EachOpponent, counter: CounterType::Poison }` |
| `chasm_skulker.rs` | Add `WhenLeavesBattlefield` (or `WhenDies`) trigger creating X 1/1 blue Squid tokens with islandwalk; X = +1/+1 counters via LKI. | `EffectAmount::CounterCount { target: EffectTarget::Source, counter: CounterType::PlusOnePlusOne }` |
| `krenko_mob_boss.rs` | Add `Activated { cost: { requires_tap: true }, effect: CreateToken { spec: goblin_11_red_with_count(EffectAmount::PermanentCount { filter: TargetFilter::with_subtype("Goblin"), controller: PlayerTarget::Controller }) }}`. | `EffectAmount::PermanentCount { filter: { subtypes: ["Goblin"] }, controller: PlayerTarget::Controller }` |
| `izoni_thousand_eyed.rs` | Add `WhenEntersBattlefield` ETB trigger. **Primary mechanic** — token half only. The "{B}{G}, sacrifice another creature" ability remains TODO (out of scope per primitive-wip.md). | `EffectAmount::CardCount { zone: ZoneTarget::Graveyard { owner: PlayerTarget::Controller }, player: PlayerTarget::Controller, filter: Some(TargetFilter::with_card_type(Creature)) }` |

**Yield: exactly 4** (matches the primitive-wip cards-affected estimate of 4).

Per `feedback_pb_yield_calibration.md` (50–65% discount for EffectAmount-style PBs): the
upper-bound estimate of 4 confirmed and the AC requires ≥ 2 — passing the gate with margin.

### Stop-and-flag triggers — walked

1. ❌ **TokenSpec used in non-spell context (e.g. dungeon completion)** — addressed at Site 6.
   Mechanical migration: `EffectAmount::Fixed(2)` for the dungeon's `spec.count = 2`.
2. ❌ **`apply_token_creation_replacement` boundary takes u32** — preserved; resolve dynamic
   count BEFORE calling it (Site 5 unchanged). Verified.
3. ❌ **`make_token` per-iteration loop currently uses `0..spec.count`** — replaced with
   `0..resolved` (Sites 3, 4). Verified.
4. ❌ **Predefined helpers (`treasure_token_spec`, etc.) keep `count: u32` API** — convert
   internally to `EffectAmount::Fixed(count as i32)`. Verified.
5. ❌ **Yield ≥ 2** — confirmed 4. Headroom for one card to slip without violating AC 3725.
6. ❌ **Walked every dispatch site** — Sites 1-15 enumerated with file:line.
7. ❌ **Hash bump 13 → 14 + sentinel sweep** — Site 11 + Site 12; 5 sentinel files identified.
8. ✅ **Anim Pakal NOT widened** — separate trigger-filter primitive (non-Gnome attacker
   filter). Append to `memory/primitives/pb-retriage-CC.md` as a new STOP-AND-FLAG seed.
9. ✅ **Izoni's secondary "sacrifice-another" ability NOT widened** — separate primitive
   (sacrifice-other cost). Leave the second TODO comment intact.

### Deferred items appended to `pb-retriage-CC.md`

The runner appends two new seeds:

```
- Anim Pakal, Thousandth Moon — non-Gnome attacker trigger filter
  Primitive needed: TargetFilter / TriggerFilter for "with one or more
  non-{subtype} creatures." Currently no DSL for negative-subtype combat trigger
  filters. Filed by PB-TS planner 2026-04-30.

- Izoni, Thousand-Eyed (secondary ability) — sacrifice-another-creature cost
  Primitive needed: ActivationCost variant for "sacrifice another creature" with
  filter exclusion of self. Currently activated abilities can require sacrifice_self
  (whole) or sacrifice_filter (any), but not "another." Filed by PB-TS planner
  2026-04-30.
```

---

## Step 5 — Hash strategy

**Current**: `HASH_SCHEMA_VERSION = 13` (PB-CC-C-followup).

**Bumped**: `HASH_SCHEMA_VERSION = 14`.

**Rationale**: `TokenSpec.count` field type changes from `u32` to `EffectAmount`. The serde wire
format changes accordingly: a `count` of 1 was previously a u32 byte sequence; it is now a
tagged-enum encoding of `EffectAmount::Fixed(1)`. Pre-PB-TS replays will not deserialize
correctly under the new format. Per `memory/conventions.md` "Hash bump rule: bump on every
change to a serialized type's field shape or variant shape. Default action: bump."

**Sentinel-assertion file sweep** (5 files):

| File | Line | Action |
|---|---|---|
| `crates/engine/tests/primitive_pb_cc_a.rs` | 99-105 | Rename `test_hash_schema_version_after_pb_cc_c_followup` → `..._after_pb_ts`. Update assertion `13` → `14u8`. Update message: `"PB-TS bumped HASH_SCHEMA_VERSION 13→14 (TokenSpec.count: u32 → EffectAmount, CR 111.1 / 608.2h)…"`. |
| `crates/engine/tests/primitive_pb_cc_c_followup.rs` | 394 | Same rename + bump + message. |
| `crates/engine/tests/pbt_up_to_n_targets.rs` | 404 | Rename `test_pbt_hash_schema_version_is_13` → `..._is_14`. Update assertion + message. |
| `crates/engine/tests/pbt_up_to_n_targets.rs` | 864 | Rename `..._sentinel_is_13_regression` → `..._sentinel_is_14_regression`. |
| `crates/engine/tests/effect_sacrifice_permanents_filter.rs` | 134 | Rename `test_sft_hash_schema_version_is_13` → `..._is_14`. |

**Hash arms requiring change**:
- `HashInto for TokenSpec` (lines 4290-4309): NO change — `self.count.hash_into(hasher)` already
  dispatches through `EffectAmount::hash_into` automatically.
- `HashInto for EffectAmount`: NO change — discriminants 0-16 are stable.
- `HASH_SCHEMA_VERSION` bumped to 14; history entry 14 appended.

---

## Step 6 — Test plan (5 mandatory tests, criterion 3724)

**File**: `crates/engine/tests/primitive_pb_ts.rs` (new file, mirrors
`primitive_pb_cc_a.rs` and `primitive_pb_cc_c_followup.rs`).

### Test (a) — Existing token-creation regression (NO test code change)

**Test name**: relies on existing tests in
`crates/engine/tests/{blood_tokens,clue_tokens,food_tokens,treasure_tokens,token_damage_search_replacement,tapped_and_attacking}.rs`.

**Action**: confirm these tests pass UNMODIFIED after PB-TS engine changes (other than the
sentinel-assertion bump in `blood_tokens.rs` line 812/826 which compares the FIELD VALUE — not
the schema version — so it follows from the field-type migration, not from new test logic).

**Verification command**: `cargo test --test blood_tokens --test clue_tokens --test food_tokens --test treasure_tokens --test tapped_and_attacking`.

**Discriminating assertion**: the existing static-count token paths produce identical token
counts before and after the migration. If a regression appears, the resolve_amount call in
`Effect::CreateToken` is wrong (likely the negative-clamp or i32→u32 conversion).

### Test (b) — Dynamic count from `EffectAmount::Fixed(N)` produces N tokens (sanity)

**Test name**: `test_create_token_with_effectamount_fixed_n_produces_n_tokens`

**CR citation**: CR 111.1 (token creation); CR 608.2h (resolution-time evaluation).

**Setup**:
1. Build a 2-player game; p1 controls a "Source" creature.
2. Construct a `TokenSpec` with `count: EffectAmount::Fixed(3)` (1/1 colorless Citizen).
3. Execute `Effect::CreateToken { spec }` directly via `execute_effect`.
4. **Assert** exactly 3 tokens of name "Citizen" appear on the battlefield owned by p1.
5. **Assert** the source's `last_created_permanent` field is set (sanity that the loop ran).

**Discriminating assertion**: a count of `Fixed(3)` produces 3 tokens, not 1 (would catch a
hardcoded `1`) and not 0 (would catch a `.max(0)` on a positive value).

### Test (c) — Krenko-style: `PermanentCount{Goblin, Controller}` scales with battlefield

**Test name**: `test_create_token_count_scales_with_permanent_count_filter`

**CR citation**: CR 111.1; CR 608.2h ("answer determined only once, when the effect is
applied"); CR 113.7a (source identity).

**Setup**:
1. Build a 2-player game; p1 controls Krenko-like Source (Goblin Warrior, included in
   "Goblin" count).
2. Add 2 more Goblin tokens manually to p1's battlefield (so 3 Goblins total counting Krenko).
3. Construct a `TokenSpec` with `count: EffectAmount::PermanentCount { filter: subtypes Goblin, controller: PlayerTarget::Controller }`.
4. Execute `Effect::CreateToken { spec }`. **Assert** 3 new Goblin tokens enter the battlefield.
5. Add another Goblin to the battlefield; execute again. **Assert** 4 new tokens this time.
6. **Discriminating assertion**: re-execution sees the post-mutation count (4) — confirms the
   resolve-at-execution semantic, not a cached value.

### Test (d) — Chasm-Skulker-style: `CounterCount{Source, P1P1}` resolves from LKI

**Test name**: `test_create_token_count_resolves_from_lki_counters_after_source_dies`

**CR citation**: CR 113.7a (LKI on zone change); CR 122.1, 122.6 (counters preserved on zone
change); CR 608.2h.

**Setup**:
1. Build a 2-player game; p1 controls a "Skulker" creature with 4 +1/+1 counters.
2. Place a death-trigger ability on Skulker via the actual `chasm_skulker.rs` card def, OR
   manually queue a `PendingTrigger` whose `effect` is `CreateToken { spec: { count: CounterCount{Source, PlusOnePlusOne} } }` and `source` is the Skulker's ObjectId.
3. Move the Skulker to graveyard via `move_object_to_zone` (mimics death). Verify the LKI
   GameObject in graveyard zone retains its `counters` map (Toothy precedent at
   `toothy_imaginary_friend.rs:43-58`).
4. Resolve the trigger via `execute_effect` with `ctx.source = skulker_id`.
5. **Assert** exactly 4 Squid tokens are created on p1's battlefield.
6. **Assert** the Skulker's GameObject in graveyard still has 4 +1/+1 counters (counters
   were not consumed by the resolution).

**Discriminating assertion**: counter-derived count = 4 (not 0, which would catch a missing
LKI counter access; not 1, which would catch a default-fallback bug).

### Test (e) — Hash determinism + `HASH_SCHEMA_VERSION` sentinel

**Test name**: `test_pb_ts_hash_schema_version_after_token_spec_count_dynamic` +
`test_pb_ts_token_spec_dynamic_count_hashes_distinctly`.

**CR citation**: hash infrastructure (no specific CR).

**Setup**:
1. **Sentinel**: `assert_eq!(HASH_SCHEMA_VERSION, 14u8, "PB-TS bumped HASH_SCHEMA_VERSION 13→14 (TokenSpec.count: u32 → EffectAmount, CR 111.1 / 608.2h)")`.
2. **Determinism**: build two `TokenSpec`s with identical fields (including `count: EffectAmount::CounterCount{Source, P1P1}`). Hash both via `state.public_state_hash()` (or call `spec.hash_into(&mut Hasher)` directly through the trait). Assert hashes equal.
3. **Distinct counts**: build three TokenSpecs differing only in `count`:
   - `count: EffectAmount::Fixed(1)`
   - `count: EffectAmount::Fixed(2)`
   - `count: EffectAmount::CounterCount { target: Source, counter: PlusOnePlusOne }`
   Hash all three. Assert all three pairwise-distinct.
4. **Static-vs-dynamic Fixed-1 distinction**: the hash of a TokenSpec with `count: EffectAmount::Fixed(1)` should be DIFFERENT from any historical hash of a TokenSpec with the old `count: 1u32` byte sequence. Cannot be tested directly (we deleted the old shape), but the schema-version sentinel bump satisfies the cross-version check by definition.
5. **Variant-discriminant coverage**: hash a TokenSpec with `count: PlayerCounterCount { player: EachOpponent, counter: Poison }` (Phyrexian Swarmlord) and one with `CardCount { zone: Graveyard{Controller}, player: Controller, filter: Some(creature) }` (Izoni). Assert distinct (confirms the new arm reads inner-EffectAmount discriminants).

---

## Verification checklist

- [ ] CR rules lookups complete: 111.1, 111.4, 614.1, 614.1c, 113.7, 113.7a, 608.2h, 122.1, 122.6
- [ ] Engine architecture walk: 15 dispatch sites with file:line, current behavior, required change
- [ ] Shape decision documented (chosen Shape A; B and C rejected with reasons)
- [ ] Dispatch unification verdict: PASS (yield = 4, ≥ AC threshold of 2)
- [ ] Hash strategy: bump 13→14, sentinel sweep across 5 test files, 1 history entry
- [ ] Test plan: 5 mandatory tests (a-e), CR citations, file paths
- [ ] Stop-and-flag triggers walked (9 triggers, all addressed)
- [ ] Plan file written: `memory/primitives/pb-plan-TS.md`
- [ ] `memory/primitive-wip.md` updated: planner checklist boxes ticked, `phase: plan-complete`

## Implementation order (for runner)

1. **Engine change 1** — Modify `TokenSpec` field shape: `card_definition.rs:3114` change
   `count: u32` to `count: EffectAmount`. Update `Default` impl at line 3156 to
   `count: EffectAmount::Fixed(1)`.
2. **Engine change 2** — Predefined helpers: update `treasure_token_spec`, `food_token_spec`,
   `clue_token_spec`, `blood_token_spec`, `army_token_spec`, `zombie_decayed_token_spec` at
   `card_definition.rs:3170-3388`. Convert `count: count` → `count: EffectAmount::Fixed(count as i32)` (or `Fixed(1)` for army_token_spec). Keep the helper signatures' `count: u32` param.
3. **Engine change 3** — `Effect::CreateToken` at `effects/mod.rs:540-585`. Insert
   `let resolved = resolve_amount(state, &spec.count, ctx).max(0) as u32;` before the
   replacement call; pass `resolved` to `apply_token_creation_replacement` instead of
   `spec.count`.
4. **Engine change 4** — `Effect::CreateTokenAndAttachSource` at `effects/mod.rs:595-660`.
   Same dynamic-count resolution. **Optional** (flag in commit message): add the missing
   `apply_token_creation_replacement` call here if behavior matches Site 4 audit; otherwise
   leave the absence intact and file as a separate seed.
5. **Engine change 5** — `dungeon.rs` 5 helper bodies + line 372 inline mutation: convert
   literal `count: 1` and `spec.count = 2` to `EffectAmount::Fixed(N)`.
6. **Engine change 6** — `tests/tapped_and_attacking.rs:37` test fixture: `count: 2` →
   `EffectAmount::Fixed(2)`. Add `EffectAmount` import.
7. **Engine change 7** — `tests/blood_tokens.rs:812, 826` sentinel assertions: switch comparison
   from `1` to `EffectAmount::Fixed(1)`, etc.
8. **Engine change 8** — `state/hash.rs:67` bump `HASH_SCHEMA_VERSION` to 14. Append history
   entry 14 (text drafted in Step 5).
9. **Engine change 9** — Sentinel sweep: 5 files in Step 5 table. Rename functions, update
   assertions, update messages.
10. **Card def 1** — Re-author `phyrexian_swarmlord.rs`. Add upkeep triggered ability creating
    a 1/1 green Phyrexian Insect creature token with infect; count =
    `EffectAmount::PlayerCounterCount { player: EachOpponent, counter: Poison }`. Token spec
    is hand-constructed (no predefined helper for "Phyrexian Insect"). Clear the TODO at
    lines 19-22.
11. **Card def 2** — Re-author `chasm_skulker.rs`. Add `WhenLeavesBattlefield` (or
    `WhenDies`; verify which is correct for "When Chasm Skulker dies") triggered ability
    creating X 1/1 blue Squid creature tokens with islandwalk; count =
    `EffectAmount::CounterCount { target: Source, counter: PlusOnePlusOne }`. Token spec is
    hand-constructed (Squid with islandwalk keyword). Clear the TODO at lines 30-31.
12. **Card def 3** — Re-author `krenko_mob_boss.rs`. Add tap-activated ability creating X 1/1
    red Goblin creature tokens; count =
    `EffectAmount::PermanentCount { filter: subtypes Goblin, controller: Controller }`.
    Token spec is hand-constructed (Goblin creature). Clear the TODO at lines 14-17.
13. **Card def 4** — Re-author `izoni_thousand_eyed.rs`. Add ETB triggered ability creating
    X 1/1 black-and-green Insect creature tokens; count =
    `EffectAmount::CardCount { zone: Graveyard{Controller}, player: Controller, filter: Some(card_type Creature) }`.
    Clear the *primary mechanic* TODO. **Leave the second TODO** ("sacrifice another creature"
    cost) intact and update its comment to reference the new pb-retriage-CC.md seed line.
14. **OOS append** — `memory/primitives/pb-retriage-CC.md`: append two new seeds (Anim Pakal
    non-Gnome attacker filter; Izoni sacrifice-another cost) per Step 4 verdict.
15. **Tests (a-e)** — 5 tests in new file `tests/primitive_pb_ts.rs`.
16. **Gates** — `cargo build --workspace`; `cargo test --workspace`; `cargo fmt --check`;
    `cargo clippy --all-targets -- -D warnings`. Test count must rise from 2720 → ≥ 2724
    (4 new tests minimum; (a) is the regression check on existing tests). All gates green
    before signaling ready.

---

## Risks & edge cases

1. **Negative resolution**: `resolve_amount` returns `i32`. `EffectAmount::Sum(Fixed(-3), Fixed(1))`
   could resolve to `-2`. The clamp `.max(0) as u32` is mandatory at every call site. Verified
   in Sites 3 and 4. **Test coverage**: not explicitly required by the (a-e) plan but worth a
   smoke check during impl — runner should manually test a `Sum(Fixed(0), Fixed(-1))` case to
   confirm 0 tokens, not panic.
2. **Replacement-effect doubling order**: critical that the resolution happens BEFORE
   `apply_token_creation_replacement`. CR 614.1 + 614.1c: replacement watches for the would-
   create-tokens event, which already has a count baked in. If the order were reversed
   (apply replacement on the EffectAmount expression), Doubling Season would not work. Verified.
3. **LKI death-trigger via `EffectTarget::Source` for Chasm Skulker**: relies on
   `move_object_to_zone` preserving the GameObject's `counters` map. The Toothy precedent at
   `toothy_imaginary_friend.rs:43-58` proves `EffectAmount::CounterCount { target: Source,
   counter: PlusOnePlusOne }` resolves correctly from a graveyard'd source. For PB-TS, the
   path goes through `resolve_amount` (not `resolve_cda_amount`), which calls
   `resolve_effect_target_list` to convert `EffectTarget::Source` to `ctx.source`. Verified.
4. **`enters_attacking` (CR 508.4) interaction**: the attack-target lookup happens once per
   token-creation effect (line 553 — outside the `for` loop). Token count is independent.
   Verified. tapped_and_attacking.rs test should still pass (Site 8 migration).
5. **`Effect::Investigate` and similar Clue/Treasure paths**: `Investigate` uses
   `clue_token_spec(1)` then loops `for _ in 0..n` where `n` comes from a separate
   `EffectAmount` (lines 661-686). The clue_token_spec helper now stores
   `count: EffectAmount::Fixed(1)`, but `Investigate` does NOT call `Effect::CreateToken` —
   it bypasses the dispatch and creates tokens directly. **No change needed**, but verify the
   `make_token` call still works (it does — `make_token` ignores `spec.count`; only
   `Effect::CreateToken` reads it).
6. **Living Weapon / `CreateTokenAndAttachSource` missing replacement-effect call** (Site 4):
   today the code does NOT apply token-doubling replacements. This is a **pre-existing bug**
   (Doubling Season + Living Weapon should produce 2 Germ tokens with the Equipment attaching
   to the first). The runner SHOULD audit this during impl: search for tests like
   "test_living_weapon_doubling_season"; if absent, leave the bug intact and append a new
   seed to pb-retriage-CC.md. If present, fix during PB-TS as a "while we're here." **Default:
   stop-and-flag, file separate seed**.
7. **Replay-viewer / TUI exhaustive matches**: per `MEMORY.md` 50%-miss-rate gotcha. TokenSpec
   field changes do NOT add new variants, but `cargo build --workspace` is mandatory after
   impl. If a serde-derive somewhere indirectly pattern-matches on the old u32 shape, a build
   error will surface.
8. **Backward-compat replays**: pre-PB-TS GameState snapshots with `TokenSpec.count: u32` in
   the wire format will fail to deserialize. This is acceptable (per `memory/conventions.md` —
   hash version bump signals incompatibility). No migration shim is required; the test corpus
   does not snapshot GameState containing stack TokenSpec serializations across the boundary.
9. **`PartialEq` change for `TokenSpec`**: `EffectAmount` derives `PartialEq` via
   `card_definition.rs:2246`, so the field change preserves struct-level equality. The
   `blood_tokens.rs:812` sentinel works with `assert_eq!(spec.count, EffectAmount::Fixed(1))`
   directly.
10. **Card def #2 trigger condition**: oracle text is "When Chasm Skulker dies." The DSL has
    both `WhenDies` (preferred for self-death) and `WhenLeavesBattlefield`. Verify by reading
    `TriggerCondition` enum during impl; pick whichever the existing card defs use for
    self-death triggers. Toothy uses `WhenLeavesBattlefield` because the oracle reads "leaves
    the battlefield"; Chasm Skulker says "dies," which is a stricter event. If both exist,
    `WhenDies` is correct.
