# PB-DX7 — execution notes, measurements and revert matrix

Task `scutemob-207`, branch `feat/pb-dx7-sr-19-gate-holes-the-hashed-field-gate-skips-path-qua`.
v3 queue rank 9. **Test-only batch.** Seeds: `OOS-DP7-11`, `OOS-DP9-13` (primary),
`OOS-DP10-1`, `OOS-DP9-10` residual (riders).

Every number below was **executed**, not predicted. Where a measurement contradicts the brief or a
seed row, the correction is stated here and carried back into the registry row.

---

## 1. Baseline

`cargo test --workspace --no-fail-fast` to a file, on this branch **before any edit**:

| | |
|---|---|
| passed | **4,508** |
| failed | **0** |
| ignored | **5** |
| result-producing targets | **46** |
| residual list | **empty** |

Matches CLAUDE.md's PB-DX26 pin exactly.

---

## 2. Both holes reproduced at HEAD (the closure standard: executed, not argued)

### 2.1 OOS-DP7-11 — the path-qualified struct skip

Deleted `self.is_token.hash_into(hasher);` from
`impl HashInto for crate::state::game_object::MergedComponent` (`hash.rs:2351`), a **live** field
of a **hashed** struct.

```
cargo test -p mtg-engine --test core hash_schema
→ test result: ok. 21 passed; 0 failed; 0 ignored
```

**Every gate green with a live field silently dropped from the state hash.** Note the second half,
which the seed row does not state: `stream_fingerprint_is_pinned` **also** stayed green — the
canonical fixture carries no merged component, so the stream digest does not cover this struct
either. The seed's "the `stream_fingerprint` would catch it only if the canonical fixture happens
to populate that variant" caveat (written about the enum half) applies to this struct half too.

### 2.2 OOS-DP9-13 — the enum-variant drop

Rewrote `EffectChoiceQuestion::SearchLibrary { candidates, may_fail_to_find }` as
`{ candidates, .. }` and dropped the `may_fail_to_find.hash_into(hasher);` feed.

```
cargo test -p mtg-engine --test core hash_schema
→ test result: ok. 21 passed; 0 failed; 0 ignored
cargo clippy -p mtg-engine --lib -- -D warnings
→ Finished (clean)
```

Green on both. The `..` silences `unused_variables`, exactly as filed.

---

## 3. Census re-derived at HEAD (dispatch hygiene 6 — the site list is a FLOOR)

**Derivation rule**: every `^impl HashInto for <T>` in `crates/engine/src/state/hash.rs`, `<T>` the
maximal `[A-Za-z0-9_:]` token; classified by resolving `<T>`'s **last `::` segment** against
`struct`/`enum` declarations under `SCAN_ROOTS` (`crates/engine/src`, `crates/card-types/src`).

| | count |
|---|---|
| `impl HashInto for` occurrences (raw grep) | 146 |
| …of which are inside comments | 7 |
| **real impls** | **139** (139 unique targets — no duplicates) |
| classified **struct** | **52** |
| classified **enum** | **79** |
| primitive/std (`u8 u32 u64 i32 usize bool String str`) | 8 |
| **path-qualified** (contain `::`) | **15** = **5 structs + 10 enums** |

The 5 path-qualified structs — `MergedComponent`, `FlashGrant`, `PlayFromTopPermission`,
`PlayFromGraveyardPermission`, `SacrificedCreatureLki` — match the seed's list exactly, and the
`↻ scutemob-159` re-verification's correction of "9 enums" to **10** is confirmed.

### 3.1 Three corrections the census forces

1. **The brief's source cite is wrong.** It puts the skip at `hash_schema.rs:1540-1541`; that
   range is the `COVERAGE_MUST_INCLUDE` assertion inside `coverage_scanners_are_not_vacuous`. The
   actual silent skip is the `let Some(body) = bodies.get(ty) else { continue };` inside
   `every_hashed_struct_field_is_hashed_or_allowlisted`. The brief flagged its own cite as a
   snapshot and asked for re-verification by symbol; it was wrong about the line, not the mechanism.
2. **"10 path-qualified enums are outside the gate for this separate reason" is a true statement
   about a misleading subset.** *All 79* hashed enums are outside the struct gate — path
   qualification has nothing to do with the enum half. Scoping the enum work to the 10 would have
   left 69 hashed enums uncovered while the batch reported OOS-DP9-13 closed. The enum work is
   scoped to all 79.
3. **The enum-shape survey found no `_ =>` arms and one `..`, and the `..` is a false positive** —
   it is inside a `TriggerCondition` comment and disappears under `strip_comments`. Seven enum
   impls (`Color`, `ManaColor`, `SuperType`, `CardType`, `Phase`, `Step`, `EffectLayer`) have no
   `match self` at all; all seven are `(*self as u8)`, a form Rust permits only for all-unit enums.

---

## 4. Riders

### 4.1 OOS-DP10-1 — the by-value cross-check (CLOSED, and strengthened)

The canonical walk is **byte-unchanged** since PB-DP10's review-fix commit `0d4adcb5`. Two later
commits touched `decision_site_walk.rs` — `87594d08` (ENG-1) and `cf89a213` (PB-DX25c) — and both
moved **ROW metadata only** (`discard_cards`'s class, `change_targets`'s prose). Neither is one of
PB-DP9's three rows. Measured by value, both walks agree exactly today:

| target | copy (key-only walk) | canonical (unit-aware walk) |
|---|---|---|
| `SearchLibrary` | 74 | **74** |
| `Scry` | 15 | **15** |
| `Surveil` | 8 | **8** |

**But the cross-check was weaker than the seed claims.** `canonical_walk_reproduces_pb_dp9_rosters`
asserts **floors** (73 / 15 / 8), not agreement, and `search_library`'s floor sits **one below** the
live count. A one-def divergence between the copy and the canonical walk passes both tests in
silence: the "cross-check" was checking that each walk was individually plausible, not that they
agreed. Closed by a new `decision_gate::pb_dp9_roster_walks_agree_by_value` that asserts
**equality** per target against a byte-faithful replica of the copy's key-only walk, with a
discriminating control (the two walks must **disagree** on a unit variant, or the equality half is
trivially true).

Promoting the walk to a shared home remains out of scope (it is an engine change, per the seed).

**`Effect::TheRingTemptsYou` was tried as a second control and dropped**, with the reason stated
in-source rather than left in as a passing-looking assertion: it is carried by **0** `Complete` defs
in the corpus, so it compares 0 to 0 and discriminates nothing. The control reddened on its own
first run, which is how this was found. `Proliferate` measures canonical **23** / key-only **0**.

### 4.2 OOS-DP9-10 residual — GATED, not deferred

The residual as filed is *"there is no gate for the shape"*. There is now:
`crates/engine/tests/core/unordered_iteration_ratchet.rs`.

Surface measured over the resolution path (`crates/engine/src/{rules,effects,state}`),
comment-stripped and whitespace-insensitive: **27 occurrences across 6 files.**

| file | count |
|---|---|
| `rules/replacement.rs` | 11 |
| `rules/sba.rs` | 5 |
| `rules/abilities.rs` | 5 |
| `effects/mod.rs` | 3 |
| `rules/commander.rs` | 2 |
| `rules/engine.rs` | 1 |

Every unlisted file under those roots is pinned at **zero** — the load-bearing half, since a table
listing only today's files is a gate that checks the channel it was written for.
`crates/engine/src/state/` measures 0 and is entirely zero-pinned. `crates/engine/src/testing/`
is deliberately excluded (20 containers in the replay harness and script schema; test
infrastructure, not resolution).

All 27 were **re-inspected at HEAD, not assumed from the seed's summary**, and every one is still
clean:

- `replacement.rs` is the only file that genuinely iterates a set to an outcome, and all three
  `already_applied.into_iter()` sites collect into a `Vec` and `sort_by_key(|id| id.0)` immediately
  (`:981`, `:1009`, `:1587` — the PB-DP9 fix-cycle repair, with the source comments saying so). The
  five `.iter().copied().collect()` sites read the `Vec` field *into* a set (construction, not
  unordered iteration), and inside `find_applicable` the set is `contains`-only (`:54`).
- `effects/mod.rs`'s `top_ids` is iterated at `:5958`, but only to partition into `matched_ids` /
  `unmatched_ids`, both `sort_by_key(|id| id.0)`'d before use. `target_remaps` is `insert`/`get`-only
  (the seed's original narrow claim, still true). `seen_names` is a membership filter.
- `sba.rs`, `abilities.rs`, `commander.rs`, `engine.rs` — all `contains`/`get`-only; none iterated.

**What the gate does NOT close, stated in its own module docs**: it is a source scan, so it cannot
do dataflow and cannot see an outcome reached through iteration of a container that is *already*
within its ceiling. It closes the residual as filed — the absence of any mechanism that makes a
**new** site loud — and the classification question is put to the human in the failure message.

---

## 5. Revert matrix

Every row **executed**, watched RED, then restored and watched GREEN. A row that cannot be made to
discriminate is recorded as UNDISCRIMINATED **with its reason**, never dropped.

| # | Gate | Revert applied | Result |
|---|---|---|---|
| V1 | *(HEAD, pre-fix)* struct field gate | delete `MergedComponent.is_token` feed (path-qualified impl) | **GREEN — the hole**, 21/21, incl. `stream_fingerprint_is_pinned` |
| V2 | *(HEAD, pre-fix)* enum variant coverage | `SearchLibrary { candidates, .. }`, feed dropped | **GREEN — the hole**, 21/21, and clippy `-D warnings` clean |
| V3 | `pb_dp9_roster_walks_agree_by_value` (control half) | replace the key-only replica with a call to the canonical walk | **RED** — "control failed: … found 23 … and the canonical walk found 23 … the equality assertions above are trivially true" |
| V4 | `pb_dp9_roster_walks_agree_by_value` (equality half) | delete the replica's `Array` arm so the copy drifts | **RED** — "disagree on SearchLibrary (0 vs 74)" |
| V5 | `unordered_container_surface_is_ratcheted` (zero-pin) | add a `HashMap` to `rules/layers.rs`, a file with **no** ceiling entry | **RED** — "rules/layers.rs: 1 > ceiling 0" |
| V6 | `unordered_container_surface_is_ratcheted` (ceiling) | add a `HashSet` to `rules/sba.rs`, a **listed** file | **RED** — "rules/sba.rs: 6 > ceiling 5" |
| V7 | `unordered_container_surface_is_ratcheted` (anti-rust) | convert `engine.rs`'s `HashSet` → `BTreeSet` | **RED** — "These ceilings are now loose: rules/engine.rs: 0 < ceiling 1" |

*(V5 and V6 were each first attempted by **prepending** the probe to the file, which put an item
before the `//!` module docs and failed to **compile** rather than failing the gate. Recorded
because a compile failure is not a gate failure and reading it as one would have been a false
pass — re-run by appending, which is what the table above reports.)*

### 5.1 V8+ — the hashed-field gate's own reverts (implement phase, Part A + Part B)

Every row below runs against the **post-fix** tree (Part A's bare-name-keyed
`hashinto_impl_bodies()` + `every_hashed_type_resolves_to_a_declaration` +
`every_hashed_struct_is_parsed_by_named_field_structs`, and Part B's
`named_enum_variants()` + `every_hashed_enum_variant_field_is_hashed_or_allowlisted`
+ `not_hashed_variant_fields_allowlist_has_no_dead_entries`), proving the NEW
mechanism itself discriminates — distinct from V1/V2 above, which proved the
mechanism was *absent* at HEAD. `git diff --stat crates/engine/src/state/hash.rs`
confirmed empty before and after this whole sub-matrix (every row here is a
revert against real production code, watched red, then restored — no
behavioral `hash.rs` edit survives).

| # | Gate | Revert applied | Result |
|---|---|---|---|
| V8 | `every_hashed_struct_field_is_hashed_or_allowlisted` (post-fix) | delete `self.is_token.hash_into(hasher);` from the path-qualified `MergedComponent` impl | **RED** — "MergedComponent.is_token" named, fails by name |
| V9 | `every_hashed_enum_variant_field_is_hashed_or_allowlisted` | `EffectChoiceQuestion::SearchLibrary { candidates, .. }`, feed dropped | **RED** — "struct pattern contains a `..` rest pattern"; also independently re-confirmed clippy-clean at this revert state |
| V10 | `every_hashed_type_resolves_to_a_declaration` | reintroduce the path-qualification skip in `hashinto_impl_bodies()` (`let bare = ty.clone();`) | **RED** — all 15 path-qualified impl targets named individually (5 structs + 10 enums), matching §3's census exactly |
| V11 | `every_hashed_enum_variant_field_is_hashed_or_allowlisted` | `KeywordAbility::Landwalk(lw_type)` → `Landwalk(_)`, tuple binding discarded | **RED** — "KeywordAbility::Landwalk.0: tuple field bound to `_`" |
| V12 | `every_hashed_enum_variant_field_is_hashed_or_allowlisted` | `CounterType`: collapse all 16 Unit arms into one `_ =>` catch-all (kept after the data-carrying `Custom(String)` arm — Rust makes a `_` arm placed *before* a more specific arm a hard `unreachable pattern` compile error, not a warning, so the catch-all was placed last to keep the revert compiling) | **RED** — catch-all rejected AND all 16 now-arm-less Unit variants individually reported ("has no arm in this enum's HashInto match") |
| V13 | `not_hashed_variant_fields_allowlist_has_no_dead_entries` (bogus entry) | `NOT_HASHED_VARIANT_FIELDS = &[("CounterType","Custom","5",…)]` — `Custom` is a 1-tuple, index 5 does not exist | **RED** — "declares no such field (dead entry — remove it or fix the name/index)" |
| V14 | `not_hashed_variant_fields_allowlist_has_no_dead_entries` (dead entry) | `NOT_HASHED_VARIANT_FIELDS = &[("CounterType","Custom","0",…)]` — field 0 (bound as `s`) IS hashed | **RED**, after fixing a real bug this row exposed (below) — "`CounterType::Custom`'s arm DOES reference `s`" |
| V15 | non-vacuity floor (`enums_checked >= MIN_HASHED_ENUMS`) | force `decl_is_enum` to always return `false`, so the main loop's `if !decl_is_enum(decl) { continue; }` skips every enum, leaving `violations` **vacuously empty** | **RED** — "only 0 hashed enums were checked (expected >= 52); … a scanner broke" — confirms the FLOOR fires, not the (vacuously-passing) violations check |

**V14 found a real bug in the dead-entry checker itself, before it ever shipped.**
The first draft resolved a tuple-variant allowlist entry's binding as
`field.to_string()` — the literal index string `"0"` — instead of re-deriving
the actual LOCAL pattern binding (`s`) from the arm's own pattern text. That
made `body_references_token(arm_body, "0")` search for a token that never
occurs, so the guard passed **GREEN when it should have failed RED** —
exactly the false-negative class this whole batch exists to close, caught
here by insisting the row itself demonstrate red before being counted.
Fixed by re-parsing the arm pattern's payload (`split_pattern` +
`split_depth0_commas`) to resolve the entry's declared field (name for
`Named`, positional index for `Tuple`) to the actual local identifier bound
at that position, for both variant shapes — then V14 reddened correctly.

All 8 of V8–V15 restored immediately after each revert; `git diff --stat`
confirmed clean (both the engine crate and the test file) before moving to
the next row. **All 8 rows discriminate; none is UNDISCRIMINATED.**

---

## 6. Discriminant-collision measurement (spec §3, "report, do not gate unless clean")

Parsed every hashed enum's `match self` arms (reusing the same
`parse_match_arms`/`top_level_match_self_body` machinery `every_hashed_enum_variant_field_is_hashed_or_allowlisted`
uses) and extracted each arm's first integer literal as its discriminant, then
checked for duplicates within each enum. **Not clean — a real, pre-existing
finding, not a false positive of the measurement method** (independently
reproduced by two implementations, a throwaway Python script and a throwaway
Rust `#[test]` against the real parser, with identical results; both were
deleted after the measurement — neither is a shipped gate, per the spec's own
"if not clean, report the exceptions" instruction).

**`Effect`'s `HashInto` impl reuses 9 discriminant values across 18 different
variants** (all other 78 hashed enums are clean — zero collisions):

| discriminant | variant A | variant B |
|---|---|---|
| 56 | `AddManaScaled` | `AddCounterAmount` |
| 57 | `AddManaRestricted` | `AdditionalCombatPhase` |
| 58 | `AddManaAnyColorRestricted` | `Fight` |
| 59 | `ChooseCreatureType` | `Bite` |
| 60 | `AddManaOfAnyColorAmount` | `CoinFlip` |
| 70 | `ExileWithDelayedReturn` | `PreventCombatDamageFromOrTo` |
| 71 | `SetReturnToHandAtEndStep` | `GainControl` |
| 73 | `AddManaFilterChoice` | `GrantPlayerProtection` |
| 74 | `BounceAll` | `PutLandFromHandOntoBattlefield` |

Verified directly in source (`hash.rs`, `impl HashInto for Effect`, lines
~6810–7139 for the first cluster and ~7100+ for the second): e.g.
`Effect::AddManaScaled { .. } => { 56u8.hash_into(hasher); .. }` at line 6815
and `Effect::AddCounterAmount { .. } => { 56u8.hash_into(hasher); .. }` at
line 7112 — both literally `56u8`, not a parsing artifact. Each "A" side is
part of a mana-family cluster (`AddMana*`) and each "B" side is an unrelated
variant (`Fight`, `CoinFlip`, `GainControl`, …); the shape strongly suggests
two historically-separate numbering sequences merged into one enum without
reconciling their ranges.

**Not gated** (the measurement did not come back clean, per spec §3's own
disposition rule) and **not fixed** — a discriminant collision inside a
`match self` producing a `Nu8` byte does not, by itself, make two DIFFERENT
`Effect` values hash identically (the subsequent field bytes differ), so this
is a hygiene/documentation-correctness defect (several `HASH_SCHEMA_HISTORY`
entries cite specific "discriminant N" values as if unique per enum) rather
than a confirmed divergence-detection blind spot — but it is real, live, and
reported to the coordinator for disposition per spec §3/§4, not silently
patched. See the final report for the full writeup.

**Superseded by §8 below** (coordinator follow-up, same session): the "does
not by itself make two Effect values hash identically" line above was an
ARGUMENT, correctly flagged by the coordinator as not a measurement — the
question is settled by executed experiment in §8, not asserted here. This
section's census (9 pairs, 18 variants, zero collisions in the other 78
enums) stands unchanged; only the "not fixed" disposition is followed up.

---

## 7. Coordinator follow-up, item 1 — `PARTIALLY_HASHED` (struct third
disposition category)

The struct gate's `body_references_field` matches `self.<field>` as a
substring regardless of what follows, so `self.on_cast_effect.is_some()
.hash_into(hasher)` passed as "covered" — the gate succeeding on a
technicality of its own matcher, identical in shape to the two holes this
batch closes.

**Census, independently re-derived** (regex over `self.(\w+)\.(\w+)\(\)\.
hash_into\(`, cross-checked against the coordinator's own count): **exactly
2** struct-half sites, both `.is_some()`:

- `PendingTrigger.embedded_effect` (`hash.rs:3561`)
- `PlayFromTopPermission.on_cast_effect` (`hash.rs:2940`)

Both `.len()`-prefix sites found by the same scan (`put_to_graveyard.len()`
in `GameEvent::LegendaryRuleApplied`, `hash.rs:4937`; `reqs.len()` in
`AbilityDefinition::Activated`'s mode-targets loop, `hash.rs:6736`) were
each individually re-inspected and confirmed followed by a SEPARATE full
element-wise iteration over the same field — full coverage, not partial,
matching the coordinator's own check.

**Additional finding beyond the coordinator's struct-scoped census, reported
but NOT acted on**: the identical `.is_some()`-summariser shape also exists
on the ENUM side, in `StackObjectKind`'s bound (non-`self.`-prefixed) arm
variables — `ActivatedAbility.embedded_effect` (`hash.rs:4093`) and
`TriggeredAbility.embedded_effect` (`hash.rs:4112`), both
`embedded_effect.is_some().hash_into(hasher)`, documented in-source at
`hash.rs:4105-4111` with the same "presence suffices, redundant with
source_object + ability_index" reasoning. A sibling variant,
`ForecastAbility.embedded_effect` (`hash.rs:4210`), DOES hash the full
value (`embedded_effect.hash_into(hasher)`). Every `Part B`
(`every_hashed_enum_variant_field_is_hashed_or_allowlisted`) currently
treats these two as covered for the same reason the struct gate did — the
token-presence matcher can't distinguish a summarised call from a full one.
This was outside the coordinator's own explicit ask (which measured
`self.<field>` sites only) and outside this batch's scope to unilaterally
extend — reported in the final report for disposition, not implemented.

**Implementation**: `StructFieldCoverage` enum (`NotReferenced` / `Full` /
`Partial(BTreeSet<method>)`), computed per-occurrence and AGGREGATED across
every whole-token `self.<field>` site in a struct's impl body — a field
with BOTH a `.len()` occurrence and a separate iteration occurrence
classifies `Full` (at least one non-summary occurrence), matching the
coordinator's own confirmed reading. New `PARTIALLY_HASHED: &[(&str, &str,
&str)]` allowlist (2 entries, each quoting the real in-source/history
justification verbatim rather than inventing one), a new dead-entry guard
(`partially_hashed_allowlist_has_no_dead_entries`), and controls added to
`coverage_scanners_are_not_vacuous` (isolated positive/negative fixtures
plus the two real entries re-classified from the actual impl bodies).

Revert proofs, appended to §5.1's table (both against real production code,
watched red, then restored):

| # | Gate | Revert applied | Result |
|---|---|---|---|
| V16 | `every_hashed_struct_field_is_hashed_or_allowlisted` (post-PARTIALLY_HASHED) | remove the `PendingTrigger.embedded_effect` entry from `PARTIALLY_HASHED` | **RED** — "PendingTrigger.embedded_effect — PARTIAL coverage only: every occurrence is `self.embedded_effect.{is_some}(..).hash_into(..)` … and it is not on PARTIALLY_HASHED" |
| V17 | `partially_hashed_allowlist_has_no_dead_entries` | convert `PlayFromTopPermission.on_cast_effect`'s site in `hash.rs` from `self.on_cast_effect.is_some().hash_into(hasher)` to a bare `self.on_cast_effect.hash_into(hasher)` (full coverage), leaving the allowlist entry in place | **RED** — "`PlayFromTopPermission::on_cast_effect` is now FULLY hashed — remove this entry (dead)." |

Both restored immediately; `git diff --stat crates/engine/src/state/hash.rs`
confirmed empty before and after (V17's revert touched `hash.rs`
production code, restored to the exact original 3-line comment + summarised
feed).

---

## 8. Coordinator follow-up, item 2 — discriminant collisions settled by
experiment, and ratcheted

The coordinator correctly rejected §6's "subsequent field bytes differ" as
an argument, not a measurement — the exact class `OOS-SIM2-6` sat behind for
4.5 months. Settled by executed experiment:

**`effect_colliding_variant_digests_are_pairwise_distinct`** (shipped,
not throwaway): constructs one plausible value of each of the 18 variants
in §6's table (defaults/simple values, not deliberately distinguishing
picks — e.g. `EffectTarget::DeclaredTarget { index: 0 }` for every
target-shaped field, `ManaColor::White`, `EffectAmount::Fixed(1)`), hashes
each through the REAL `HashInto` impl (`blake3::Hasher`, the same type
`public_state_hash` uses), and asserts all 18 resulting 32-byte digests are
pairwise distinct. **Result: GREEN — no collision found among the 18
sampled values.** The test's own doc states the limit explicitly: this is
evidence from one sampled point per variant, not a proof of injectivity
over the whole field-value space.

**`discriminant_collisions_are_ratcheted_at_their_known_bad_state`**
(shipped): a NEW scanner (`enum_discriminant_collisions`, reusing
`top_level_match_self_body` / `parse_match_arms` / `split_pattern` from
Part B) computes, per hashed enum, every discriminant value shared by more
than one variant. `KNOWN_EFFECT_DISCRIMINANT_COLLISIONS` pins the 9 pairs
from §6 exactly (alphabetized per pair for a stable comparison); the gate
asserts `Effect`'s computed collision set equals the pin EXACTLY (not a
floor — a FIXED pair silently disappearing would mean the pin is stale and
must be edited in the same commit as whatever fixed it) and that **no other
hashed enum** has any collision at all — `Effect` is the one pinned
exception, not a template for others.

**Numbering-sequence-merge note, verified precisely** (coordinator asked
this be recorded): the `AddMana*` family sits at `hash.rs:6832-6875`
(`AddManaFilterChoice`=73, `AddManaScaled`=56, `AddManaRestricted`=57,
`AddManaAnyColorRestricted`=58, `AddManaOfAnyColorAmount`=60) and a second,
unrelated cluster sits at `hash.rs:7130-7388` (`BounceAll`=74,
`AddCounterAmount`=56, `AdditionalCombatPhase`=57, `Fight`=58, `Bite`=59,
`CoinFlip`=60, `ExileWithDelayedReturn`=70, `PreventCombatDamageFromOrTo`=70,
`GainControl`=71, `GrantPlayerProtection`=73,
`PutLandFromHandOntoBattlefield`=74) — both clusters independently
numbered from a low starting point, confirming the two-sequences-merged
reading. In-source comments (`// CR 122: AddCounterAmount (discriminant
56)`, etc.) document these as if unique; several `HASH_SCHEMA_HISTORY`
entries inherit that error. **The documentation is wrong regardless of
whether the digests collide** — not fixed here (no `hash.rs` behavioral
edit, and correcting nine in-source discriminant comments plus however many
`HASH_SCHEMA_HISTORY` prose references is a real edit with its own review
surface, left to the coordinator's disposition).

Revert proofs, appended to §5.1's table:

| # | Gate | Revert applied | Result |
|---|---|---|---|
| V18 | `effect_colliding_variant_digests_are_pairwise_distinct` | append a 19th value that's a literal duplicate of `PutLandFromHandOntoBattlefield`'s value (and bump the non-vacuity floor 18→19 so the duplicate isn't rejected before reaching the pairwise check) | **RED** — "PutLandFromHandOntoBattlefield and PutLandFromHandOntoBattlefield-REVERT-SCRATCH-DUPLICATE hash IDENTICALLY: `4e55c1b8…`" — names both, prints the shared digest |
| V19 | `discriminant_collisions_are_ratcheted_at_their_known_bad_state` (new-collision half) | `CounterType::MinusOneMinusOne`'s discriminant `1u8` → `0u8` (colliding with `PlusOnePlusOne`'s `0u8`) | **RED** — "CounterType discriminant 0: [\"PlusOnePlusOne\", \"MinusOneMinusOne\"]" |
| V20 | `discriminant_collisions_are_ratcheted_at_their_known_bad_state` (stale-pin half) | `Effect::PutLandFromHandOntoBattlefield`'s discriminant `74u8` → `200u8` (fixing the (74, BounceAll, PutLandFromHandOntoBattlefield) collision without updating the pin) | **RED** — `assert_eq!` diff shows the pinned set retains `(74, "BounceAll", "PutLandFromHandOntoBattlefield")` while the live-computed set no longer does |

All three restored immediately; `git diff --stat crates/engine/src/state/hash.rs`
confirmed empty before and after (V19/V20 touched `hash.rs` production
code).

---

## 9. Coordinator follow-up, item 3 — `GameState` → `public_state_hash`
field coverage

The struct gate's own module doc explicitly carves `GameState`'s selective
hash functions out of scope (correctly — "every field is hashed" would be
the wrong rule for a function that selects fields on purpose), but named no
mechanism keeping the SELECTION honest. `SR-17`'s `decl_fingerprint` forces
a look on a 46th field, but its prompt is "a fingerprint moved", not "you
added a field and did not feed it".

**Field count re-derived independently, not trusted from the coordinator's
own corrected number**: reused `named_field_structs()` (already proven
correct for the crate::/imbl:: path-prefix pitfall, since Part A's coverage
already exercises it on other structs) against `GameState`'s declaration —
**45 fields**, confirmed by printing the full list. `public_state_hash`'s
body (extracted the same way `hashinto_impl_bodies()` extracts an impl
body) references **42** of them by the existing `body_references_field`
matcher; missing: `loop_detection_hashes`, `history`, `card_registry` —
matching the coordinator's corrected count exactly.

**Each of the three independently investigated, not accepted from the
field's own doc comment** (the coordinator's explicit instruction: STOP if
any turns out unsound):

- **`card_registry`**: `#[serde(skip)]`, reconstructed on load. Sound —
  static card-definition data identical for every instance of a format's
  card pool, never tied to a specific game's trajectory.
- **`loop_detection_hashes`**: the field's OWN doc (`state/mod.rs`) states
  two independent engine instances processing the SAME legal game may
  accumulate DIFFERENT hash histories depending on when their
  mandatory-action sequences began. This is the STRONGEST of the three —
  including it would produce FALSE mismatches between two genuinely-agreeing
  states, not merely fail to catch real ones. Sound.
- **`history`**: the coordinator's own guess ("derived from the command
  sequence, not independent state") is not quite the reason stated in
  `hash.rs`'s own doc — the doc says "Event history (O(n) in game length)",
  a COST reason, not a redundancy one. Verified by dataflow
  (`grep -rn '\.history()'` across `crates/engine/src`,
  `crates/card-types/src`, `crates/simulator/src`, `crates/view-model/src`,
  `crates/engine/tests`, `tools/`): the ONLY caller anywhere is a single
  test assertion (`crates/engine/tests/rules/replacement_effects.rs:812`);
  ZERO rules-decision code reads it. Every existing look-back mechanic (CR
  603.10a LKI snapshots, etc.) captures its OWN dedicated field instead of
  scanning `history`, and those dedicated fields ARE hashed. So no
  rules-visible desync is currently blind to the exclusion — sound TODAY,
  with the caveat (stated in the allowlist entry itself) that this must be
  re-examined if a future look-back trigger ever reads `history()` directly.

None of the three required a STOP-and-report; all three carry their real,
independently-verified reason in `GAMESTATE_NOT_IN_PUBLIC_HASH`.

**Deliberately NOT applied to `private_state_hash`** per the coordinator's
explicit correction — it is scoped to one player's hidden zones by design,
so the same rule would be false on its own terms.

Revert proofs, appended to §5.1's table:

| # | Gate | Revert applied | Result |
|---|---|---|---|
| V21 | `every_gamestate_field_is_in_public_hash_or_allowlisted` | delete `self.timestamp_counter.hash_into(&mut hasher);` from `public_state_hash` | **RED** — "timestamp_counter" named, fails by name |
| V22 | `gamestate_not_in_public_hash_has_no_dead_entries` | add a bogus entry `("turn", "REVERT-SCRATCH…")` — `turn` IS referenced in `public_state_hash` | **RED** — "GAMESTATE_NOT_IN_PUBLIC_HASH entry (turn): public_state_hash DOES reference `turn` — remove it from the allowlist (dead entry)." |

Both restored immediately; `git diff --stat crates/engine/src/state/hash.rs`
confirmed empty before and after (V21 touched `hash.rs` production code).

---

## 10. Combined close conditions (after items 1–3)

`cargo test -p mtg-engine --test core hash_schema`: **32 / 0** (was 27
before this follow-up round; +5: `partially_hashed_allowlist_has_no_dead_
entries`, `discriminant_collisions_are_ratcheted_at_their_known_bad_state`,
`effect_colliding_variant_digests_are_pairwise_distinct`,
`every_gamestate_field_is_in_public_hash_or_allowlisted`,
`gamestate_not_in_public_hash_has_no_dead_entries` — the `PARTIALLY_HASHED`
controls landed inside the EXISTING `coverage_scanners_are_not_vacuous`,
not as a new test function).

`cargo test --workspace --no-fail-fast`: **4,523 / 0 / 5**, 46 targets,
residual empty (+5 over the 4,518 pin at the end of the original report,
matching the 5 new `#[test]` functions exactly).

`cargo clippy --workspace --all-targets -- -D warnings`: clean.
`cargo fmt --check`: clean. `tools/check-defs-fmt.sh`: clean, 1,803 defs.
`cargo build --workspace`: clean.

`HASH_SCHEMA_VERSION` unmoved at 74 (`hash_schema_version_sentinel` green).
`PROTOCOL` unmoved (17/17 `protocol_schema` green).

`git diff -- crates/engine/src/state/hash.rs | grep -vE '^[+-]//'`: still
EMPTY — every line touched in `hash.rs`, across the original batch AND this
follow-up round, is a comment. `git status --short`: exactly 3 files
(`crates/engine/src/state/hash.rs`, `crates/engine/tests/core/hash_schema.rs`,
this file) — 0 card-def edits, 0 other engine-source edits.

**Revert matrix total: 22 rows (V1–V22), all discriminate, none
UNDISCRIMINATED.**

---

## 11. Coordinator disposition round 2, item 1 — TAKE IT: `PARTIALLY_HASHED_VARIANT_FIELDS`

Coordinator's ruling: without this, the two halves of the SR-19 gate
disagreed about what counts as coverage for the SAME field name
(`embedded_effect`), the SAME `.is_some()` shape, the SAME documented
reasoning — struct-side `PendingTrigger.embedded_effect` reported Partial,
enum-side `StackObjectKind::ActivatedAbility.embedded_effect` reported Full.
Shipped.

**Refactor first, to make the two halves share one classifier rather than
two independently-maintained copies of the same logic** (the coordinator's
own stated worry — "opposite verdicts" — is exactly the risk two divergent
implementations of the same idea create): `StructFieldCoverage` renamed to
`FieldCoverage` (generic — a token's coverage, not specifically a struct
field's), with a new shared core `token_coverage(body, needle) ->
FieldCoverage`. `struct_field_coverage(body, field)` now calls
`token_coverage(body, &format!("self.{field}"))`; new
`variant_field_coverage(arm_body, binding)` calls `token_coverage(arm_body,
binding)` directly (no `self.` prefix — an enum arm binds a bare local
name). One aggregation rule (per-occurrence classify, `Partial` only if
EVERY occurrence is a summariser) now serves both.

**New `PARTIALLY_HASHED_VARIANT_FIELDS: &[(&str, &str, &str, &str)]`**,
populated with exactly the 2 known entries
(`StackObjectKind::ActivatedAbility.embedded_effect`,
`StackObjectKind::TriggeredAbility.embedded_effect`), each quoting the
real in-source reasoning (`hash.rs:4105-4111`).
`StackObjectKind::ForecastAbility.embedded_effect` deliberately has NO
entry — it hashes the full effect — and a control in
`enum_coverage_scanners_are_not_vacuous` asserts its coverage is `Full`
directly against the real impl body, making the asymmetry an executed
assertion, not just a comment claiming it.

**Re-ran the full enum scan afterward, per the coordinator's explicit
instruction, rather than assuming the 2 known entries were the total.**
A scratch (unshipped, deleted after measurement) sweep classified EVERY
variant field of EVERY hashed enum via `variant_field_coverage` and
counted every `Partial` result, allowlisted or not:

```
TOTAL PARTIAL BINDINGS: 2
  StackObjectKind::ActivatedAbility.embedded_effect (bound embedded_effect) via {"is_some"}
  StackObjectKind::TriggeredAbility.embedded_effect (bound embedded_effect) via {"is_some"}
```

**No new findings** — the total is exactly the 2 already known and
allowlisted. Reported per instruction rather than silently trusted.

New dead-entry guard
(`partially_hashed_variant_fields_allowlist_has_no_dead_entries`, mirroring
`partially_hashed_allowlist_has_no_dead_entries`'s struct-side shape) and
controls added to `enum_coverage_scanners_are_not_vacuous`.

Revert proofs, appended to §5.1's table:

| # | Gate | Revert applied | Result |
|---|---|---|---|
| V23 | `every_hashed_enum_variant_field_is_hashed_or_allowlisted` (post-`PARTIALLY_HASHED_VARIANT_FIELDS`) | remove the `ActivatedAbility` entry | **RED** — "StackObjectKind::ActivatedAbility.embedded_effect (bound as `embedded_effect`): PARTIAL coverage only … and it is not on PARTIALLY_HASHED_VARIANT_FIELDS" |
| V24 | `partially_hashed_variant_fields_allowlist_has_no_dead_entries` | convert `TriggeredAbility`'s site in `hash.rs` from `embedded_effect.is_some().hash_into(hasher)` to bare `embedded_effect.hash_into(hasher)`, leaving the entry in place | **RED** — "`StackObjectKind::TriggeredAbility`'s `embedded_effect` is now FULLY hashed — remove this entry (dead)." |

Both restored immediately; `git diff --stat crates/engine/src/state/hash.rs`
confirmed empty before and after (V24 touched `hash.rs` production code,
restored to the exact original 8-line comment + summarised feed).

---

## 12. Coordinator disposition round 2, item 2 — discriminant documentation corrected

Comment-only, `hash.rs`. Two parts:

**(a) A new header block immediately above `impl HashInto for Effect`**
states the true situation: the leading `Nu8` per arm is an ARM TAG, not a
unique identifier; lists the 9 reused-tag pairs (from §6/§8); names the
two-clusters-merged shape with approximate line ranges; and states that
uniqueness of the tag was never what made the byte STREAM injective —
citing the executed `effect_colliding_variant_digests_are_pairwise_distinct`
result rather than arguing it.

**(b) The 14 individual arm comments that named a reused tag** (of the 18
colliding variants, 4 — `AddManaScaled`, `AddManaRestricted`,
`AddManaAnyColorRestricted`, `ChooseCreatureType` — carried no discriminant
comment at all and needed no edit) reworded from `(discriminant N)` to
`(arm tag N -- reused, see the impl header note above)` at each site,
verified individually unique in the file before editing (no
`replace_all`, no risk of touching a same-numbered comment on an unrelated,
non-colliding enum elsewhere in `hash.rs` — there are ~90 such comments
across `KeywordAbility`, `StackObjectKind`, `GameEvent`,
`AbilityDefinition`, etc. reusing the SAME numbers 56-96 for THEIR OWN,
individually-unique numbering — confirmed clean by the ratchet test, so
those were correctly left untouched).

**`HASH_SCHEMA_HISTORY` — checked, not assumed, and the coordinator's own
framing ("naming which entries inherited the error") turned out to be
UNVERIFIED for this specific set, corrected honestly rather than complied
with literally.** Searched the full history block (`hash.rs` lines ~12-780)
for every occurrence of the 9 shared tag VALUES and every occurrence of all
18 colliding variant NAMES: **zero matches, either way.** No numbered
`- N:` row anywhere in the history names any of these 18 `Effect` variants
or claims uniqueness for tags 56/57/58/59/60/70/71/73/74. So the premise
"several entries inherited the error" — stated in my own PRIOR report to
the coordinator, and echoed in the coordinator's follow-up message — does
**not** hold for this specific set once actually checked; there is nothing
row-specific to correct. **Hard constraint honored regardless**: a new,
clearly-marked, non-numbered correction paragraph was added immediately
after entry `- 74:` and before `pub const HASH_SCHEMA_VERSION`, explicitly
labeled "NOT a version bump, and NOT an edit to any row above", stating
the verified-clean result plainly (rather than inventing a target to
correct) and generalizing the caution: any FUTURE "(discriminant N)"
phrasing on an `Effect` variant, wherever it appears, should be read as
arm-tag identification, not a per-enum uniqueness guarantee. No `- N:` row
was touched; no fingerprint or epoch constant was touched.

No revert proof applies to a comment-only doc correction with no
executable assertion behind it (there is no gate that reads this prose);
recorded here as a manual verification (the two greps above, both
re-run and reproduced) rather than a red/green row.

---

## 13. Final combined close conditions (after both round-2 items)

`cargo test -p mtg-engine --test core hash_schema`: **33 / 0** (+1 over
round 1's 32: `partially_hashed_variant_fields_allowlist_has_no_dead_entries`
— the `PARTIALLY_HASHED_VARIANT_FIELDS` controls landed inside the
EXISTING `enum_coverage_scanners_are_not_vacuous`, same pattern as the
struct-side round).

`cargo test --workspace --no-fail-fast`: **4,524 / 0 / 5**, 46 targets,
residual empty. **Itemized against the 4,508 baseline: +16** = **4 already
present from the OOS-DP10-1 / OOS-DP9-10 riders** (committed on this
branch, `0c47700f`, before this session started — 1 test in
`core/decision_gate.rs`, 3 in the new `core/unordered_iteration_ratchet.rs`)
+ **12 from this session's three PB-DX7-gate-spec items and two
coordinator-directed follow-up rounds** (hash_schema.rs 21 → 33).

`cargo clippy --workspace --all-targets -- -D warnings`: clean (executed,
this round found zero new findings).
`cargo fmt --check`: clean (one pass applied — 4 whitespace-only reflow
diffs in `hash_schema.rs`'s new code, none in `hash.rs`).
`tools/check-defs-fmt.sh`: clean, 1,803 defs.
`cargo build --workspace`: clean.

**Gate-executed, not assumed:**
- `cargo test -p mtg-engine --test core hash_schema_version_sentinel`:
  **1/0**, asserting `HASH_SCHEMA_VERSION == 74` — **HASH = 74, executed
  and unmoved** across the entire task (both rounds).
- `cargo test -p mtg-engine --test core protocol_schema`: **17/0**,
  including `protocol_version_sentinel` — **PROTOCOL = 35** (read from
  `crates/engine/src/rules/protocol.rs:360`'s `PROTOCOL_VERSION` const,
  confirmed unmoved by the passing sentinel), **executed and unmoved**
  across the entire task.

`git diff -- crates/engine/src/state/hash.rs`: **171 lines changed
(+133/-38)** across the WHOLE task; verified via an executed Python
line-by-line check (not a grep pattern, after an earlier grep pattern
in this session's own final report gave a FALSE POSITIVE by not
accounting for indentation before `//`) that **zero** of those lines are
non-comment. `git status --short`: still exactly 3 files
(`crates/engine/src/state/hash.rs`, `crates/engine/tests/core/hash_schema.rs`,
this file) — 0 card-def edits, 0 other engine-source edits, across the
entire task.

**Revert matrix total: 24 rows (V1–V24), all discriminate, none
UNDISCRIMINATED.**

*(Corrected by the coordinator at close: the implement phase's own summary line
said "26 rows (V1–V26)". The table holds **24** — V1–V24 contiguous, with no V25
and no V27; the "V26" was a miscount in the summary sentence, not a missing row.
Recorded rather than silently fixed, because a count of proofs is exactly the
kind of claim this batch exists to make checkable, and it was wrong in the
document whose subject is gates that overstate their own coverage. Verified by
enumerating the table's row labels, not by re-reading the sentence.)*

---

## 15. `/review` fix cycle (2026-08-12) — `memory/primitives/pb-review-DX7.md`

**Verdict: needs-fix — 2 HIGH, 5 MEDIUM, 9 LOW. All 16 findings taken; none
disputed.** The reviewer had no shell — nothing in the review was executed.
The coordinator executed both HIGHs personally and confirmed both real
before dispatching the fix. Every fix below was re-verified by an executed
revert using the review's own named defeat, not a synthetic stand-in.

### H1 — the unordered-container ratchet counted one spelling (27), not
containers (real: 85 across 9 files, not 6)

**Fixed.** `unordered_container_count` rewritten from `Hash{Map,Set}<`
substring counting to whole-token `HashMap`/`HashSet` matching (annotation,
construction, turbofish, `use` imports — everything). Re-measured: **85**
occurrences across **9** files (`replacement.rs` 21, `effects/mod.rs` 19,
`abilities.rs` 11, `casting.rs` 9, `sba.rs` 8, `resolution.rs` 7,
`commander.rs` 4, `engine.rs` 3, `turn_actions.rs` 3). All 85 traced to a
named variable/field/parameter and classified — no new hazard, every
addition beyond the original 27 is an import, a parameter restatement, a
`.clone()`, or an empty-literal `&HashSet::new()` argument. `UNORDERED_CEILINGS`
re-pinned to the 9-file table; `MIN_TOTAL_FOUND` 20 → 60. Module doc and the
`OOS-DP9-10` registry row corrected (the "deliberately obscure code" residual
claim was itself wrong about type-inferred construction, which is 58 of the
85 real occurrences).

Coordinator's scope decision, recorded as instructed: the reviewer argued
`OOS-DP9-10` should have been deferred (different seed, different subsystem,
closed on a partial scan). Coordinator's ruling: **keep it and fix it
properly** — deferring now would leave the registry row carrying a wrong
count and a gate that green-lights the defect it names, worse than either
shipping it correct or never starting. Not pulled; the fixed ratchet is
honest (see the re-measured 85/9-file table and the re-inspected
classification above).

Revert row (continuing V-numbering, restored immediately, `git diff --stat`
confirmed clean after each):

| # | Gate | Revert applied | Result |
|---|---|---|---|
| V-H1 | `unordered_container_surface_is_ratcheted` | injected the review's exact type-inferred defeat into `rules/layers.rs` (`let mut best = std::collections::HashMap::new(); ... .max_by_key(...)`) | **RED** — "rules/layers.rs: 1 > ceiling 0" |
| V-H1b | same | ceiling-raise revert re-run post-fix: added an un-annotated `HashSet::new()` to `rules/sba.rs` (listed, ceiling 8) | **RED** — "rules/sba.rs: 10 > ceiling 8" |
| V-H1c | same | conversion revert re-run post-fix: `rules/engine.rs`'s `sources_on_bf` `HashSet` → `BTreeSet` | **RED** — "rules/engine.rs: 2 < ceiling 3" |

### H2 — `FieldCoverage::Full` meant "the token appears", not "the value is
hashed" (the `OOS-DP9-13` defect, one spelling over)

**Fixed.** `token_coverage`'s fail-open `else` arm (any non-summariser
occurrence → `Full`) removed. New `FieldCoverage::Unverified` (token present,
no occurrence matches a recognised shape) fails the gate — never silently
passes, never satisfies either `NOT_HASHED*` allowlist (their dead-entry
guards require raw textual absence, deliberately kept separate). `Full` now
requires one of: direct feed, iteration-with-hashing-body, `match`-with-
hashing-body, `if let Some(...)`-with-hashing-body (incl. a collection
`.get()`/`.get_mut()` lookup scrutinee), a cast-to-repr (`(*x as u8)` or
`(x as u64)`, with or without one intervening method call), or a bare
field/tuple-index access chain (`x.0.hash_into(`). Every shape was surveyed
against REAL `hash.rs` occurrences before being added (not invented) —
running the fixed classifier against the whole codebase surfaced 8
previously-mis-classified real fields (`DungeonState.current_room`,
`GameObject.designations`/`.cast_alt_cost`, `ModeSelection.mode_costs`/
`.mode_targets`, `PendingEffectChoice.index`, plus ~10 enum tuple fields
wrapping `SubType`, plus `GameState.objects`/`.permanents_put_into_
graveyard_this_turn`), each individually verified against its real source
and either recognised as a legitimate new shape (all of them) or would have
required a new allowlist entry (none did — zero regressions, zero new
allowlist entries). One additional bug found and fixed WHILE implementing
this fix, not shipped: the strict-adjacency checks (`.starts_with(".hash_into(")`
etc.) did not tolerate whitespace/newlines between a token and its chained
call, breaking on real rustfmt-wrapped code
(`self.permanents_put_into_graveyard_this_turn\n    .hash_into(...)`) — fixed
with a shared `skip_ws` helper applied at every adjacency check.

Revert rows (continuing V-numbering):

| # | Gate | Revert applied | Result |
|---|---|---|---|
| V-H2 | `every_hashed_enum_variant_field_is_hashed_or_allowlisted` | the review's exact defeat: `EffectChoiceQuestion::SearchLibrary`'s `may_fail_to_find.hash_into(hasher);` → `let _ = may_fail_to_find;` | **RED** — "UNVERIFIED"; independently confirmed clippy-clean at this revert state, matching the review's own observation |
| V-H2b | `every_hashed_struct_field_is_hashed_or_allowlisted` | struct-side equivalent: `PendingCleanupDiscard.count` → `let _ = self.count;` | **RED** — proves the shared classifier fix reaches the struct half too, per the coordinator's "check, don't assume" instruction |

### M3 — nothing required an arm to feed the hasher at all

**Fixed.** Every Unit-variant arm body, and the whole bare-cast-shape impl
body, must now contain at least one `.hash_into(`. Exception carved out and
verified live: the `let disc: u8 = match self { A => 0, ... };
disc.hash_into(hasher);` indirect-discriminant idiom (`AltCostKind`,
`DungeonId`) legitimately has no per-arm `.hash_into(` — detected via a new
`match_self_result_is_bound_and_hashed` body-level check. Also fixed:
`enum_discriminant_collisions` now returns arms with NO integer literal as a
reportable finding (previously silently skipped).

| # | Gate | Revert applied | Result |
|---|---|---|---|
| V-M3 | `every_hashed_enum_variant_field_is_hashed_or_allowlisted` | the review's exact defeat: `GiftType::Food => 0u8.hash_into(hasher)` → `GiftType::Food => {}` | **RED** — "this arm's body feeds NOTHING to the hasher" |
| V-M3b | `discriminant_collisions_are_ratcheted_at_their_known_bad_state` | same defeat | **RED**, independently — "GiftType::Food" reported with no integer literal |

### M4 — the Named branch did not reject `_`, the Tuple branch did

**Fixed.** Named branch now rejects a field bound to exactly `_`, mirroring
the tuple branch.

| # | Gate | Revert applied | Result |
|---|---|---|---|
| V-M4 | `every_hashed_enum_variant_field_is_hashed_or_allowlisted` | `EffectChoiceQuestion::SearchLibrary`'s pattern rewritten to `{ candidates, may_fail_to_find: _ }` with a stray `let _ = 1;` in the body (the review's exact latent-shape description) | **RED** — "named field bound to `_`" |

### M5 — the GameState gate stopped one level short: hand-hashed collection
elements with no `HashInto` impl of their own were invisible

**Fixed.** New `HAND_HASHED_ELEMENT_TYPES` roster + `hand_hashed_gamestate_
elements_cover_every_field`, covering the one genuine instance
(`AdditionalLandPlaySource`, hand-hashed in `additional_land_play_sources`'s
loop) — every other hand-destructured collection in `public_state_hash`
either unpacks a fixed-arity tuple or delegates to a type that already has
its own `HashInto` impl. Uses `token_coverage`, not bare presence — a first
draft using `body_references_token` was caught by its OWN revert proof
passing when it should have failed (the exact H2 shape, recurring inside its
own fix), corrected before shipping.

| # | Gate | Revert applied | Result |
|---|---|---|---|
| V-M5 | `hand_hashed_gamestate_elements_cover_every_field` | `src.count.hash_into(&mut hasher);` → `let _ = &src.count;` | **RED** — "declares fields never FULLY fed to the hasher" |

### M6 — the GameState gate used `body_references_field`, the matcher this
same batch diagnosed as insufficient

**Fixed.** Both GameState gates switched to `struct_field_coverage`; new
`PARTIALLY_HASHED_GAMESTATE` allowlist + dead-entry guard, shipped EMPTY
(verified: all 42 currently-referenced fields classify `Full`, zero
regressions).

| # | Gate | Revert applied | Result |
|---|---|---|---|
| V-M6 | `every_gamestate_field_is_in_public_hash_or_allowlisted` | the review's exact example: `match self.day_night {...}` → `self.day_night.is_some().hash_into(&mut hasher);` | **RED** — "PARTIAL coverage only ... is_some" |

### M7 — both `PARTIALLY_HASHED_VARIANT_FIELDS` reasons cited a line range
with no such reasoning

**Fixed.** Re-cited to the real locations (`ActivatedAbility` feed
`hash.rs:4120`, `TriggeredAbility` reasoning `hash.rs:4136-4142`,
`ForecastAbility` feed `hash.rs:4241`). `ActivatedAbility`'s arm — which
genuinely carried no comment of its own, confirmed by the review — was given
one (comment-only), mirroring its sibling, so "documented in-source" is now
literally true rather than inferred. No revert applies (a citation
correction has no executable assertion behind it); verified by re-reading
the cited line ranges directly, twice (once before, once after the ADD in
`hash.rs` shifted every subsequent line number by +4).

### LOW findings — all 9 taken

- **L1**: `loop_detection_hashes`'s allowlist entry mis-cited `state/mod.rs`;
  the real citation is `hash.rs:8290-8293` (`public_state_hash`'s own
  "Excludes:" doc). Fixed in the entry and in the `OOS-DX7-3` registry row.
- **L2**: `hash.rs:6793-6800` claimed the discriminant error was "inherited
  by several `HASH_SCHEMA_HISTORY` entries below" — wrong twice (the history
  is above, and zero entries actually reference the 18 colliding variants,
  already verified and stated correctly elsewhere in the same file). Reworded
  to match the already-correct verified result.
- **L3**: `OOS-DP9-13`/`OOS-DX7-3` both said 3 GameState fields "reached
  neither `public_state_hash` nor any stated exclusion list" — wrong for 2 of
  3 (`history`/`loop_detection_hashes` ARE named in `public_state_hash`'s own
  doc; only `card_registry` was genuinely unstated). Both registry rows
  corrected.
- **L4**: `enum_coverage_scanners_are_not_vacuous` floored ALL declared enums
  (measured: 109) against `MIN_HASHED_ENUMS` (52, meant for the 79 HASHED
  ones) — ~2x unintended slack. New `MIN_DECLARED_ENUMS = 72` against the
  real 109.
- **L5**: `hashinto_impl_bodies()` could silently drop a malformed/unusual
  spelling with `MIN_HASHINTO_IMPLS = 80` giving 59 impls of headroom before
  noticing. New `hashinto_impl_bodies_parses_every_raw_occurrence` asserts an
  EXACT match between the parsed count and an independent raw needle count.
- **L6**: `pb_dp9_roster_walks_agree_by_value` compares a REPLICA of the
  copy, not the real copy in `pb_dp9_effect_choice.rs`. Residual stated in
  both the test's own doc and the `OOS-DP10-1` registry row: proves the
  ALGORITHM is blind to unit variants, not that the real copy still IS that
  algorithm. Durable fix (shared function) confirmed out of scope, not
  silently deferred.
- **L7**: `first_integer_literal` read the first digit run ANYWHERE,
  including inside an identifier (`u32`, `x2`) if a cast preceded the real
  tag. Fixed to skip a digit run immediately preceded by an identifier
  character, WITHOUT requiring a `uN` suffix (which would have broken the
  legitimate bare-literal indirect-discriminant idiom M3 also touches).
- **L8**: trim candidate for `effect_colliding_variant_digests_are_pairwise_
  distinct`. **Coordinator's explicit instruction: keep it** — it is the
  only executed evidence behind `OOS-DX7-1`. Not trimmed.
- **L9**: `every_hashed_struct_is_parsed_by_named_field_structs`'s non-`pub`
  half cannot fire independently (`every_hashed_type_resolves_to_a_declaration`
  always catches the same case first). Documented honestly in the function's
  own doc rather than left implying independent coverage; not removed (still
  a better error message on the same underlying failure).

### Final close conditions, this round

`cargo test -p mtg-engine --test core hash_schema`: **36/0** (+2 over the
34 pin before this review round — `hashinto_impl_bodies_parses_every_raw_
occurrence` (L5) and `hand_hashed_gamestate_elements_cover_every_field`
(M5); every other fix strengthened an EXISTING test's assertion rather than
adding a new one, or was a doc/citation-only change).

`cargo test --workspace --no-fail-fast`: **4,527 / 0 / 5**, 46 targets,
residual empty. Itemized against the **4,508** baseline: **+19** = 4 already
on the branch from the pre-existing OOS-DP10-1/OOS-DP9-10 riders +
15 from `hash_schema.rs` growing 21 → 36 across the whole task (three
implementation rounds plus this review fix cycle).

`cargo clippy --workspace --all-targets -- -D warnings`: clean (one real
finding hit and fixed along the way — `clippy::if_same_then_else` on the
`field_chain_directly_hashed`/`cast_wrapped_and_hashed` two-arm `else if`
chain, merged with `||`).
`cargo fmt --check`: clean (one pass applied).
`tools/check-defs-fmt.sh`: clean, 1,803 defs.
`cargo build --workspace`: clean.

**Gate-executed, not assumed:**
- `hash_schema_version_sentinel`: **1/0**, asserting `HASH_SCHEMA_VERSION ==
  74` — **HASH = 74**, unmoved across the whole task including this review
  round.
- `protocol_schema`: **17/0** including `protocol_version_sentinel` —
  **PROTOCOL = 35** (`crates/engine/src/rules/protocol.rs:360`), unmoved.

`git diff -- crates/engine/src/state/hash.rs`: verified via the same
executed Python line-by-line check (not grep) — **zero non-comment lines**,
across the WHOLE task including this review round (the `ActivatedAbility`
comment addition for M7 is comment-only, as are the L2/M3-header corrections
already present).

`git status --short`: **5** files now (`hash.rs`, `hash_schema.rs`,
`decision_gate.rs` (L6 doc), `unordered_iteration_ratchet.rs` (H1),
`docs/audits/decision-point-audit.md` (L1/L3/L6 registry corrections)) —
0 card-def edits, 0 other engine-source edits.

**Not committed** (coordinator's instruction — they commit).
