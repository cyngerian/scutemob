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

Rows V8+ (the hashed-field gate's own reverts) are appended by the implement phase.
