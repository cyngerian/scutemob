# Primitive Batch Review: PB-RS2 — Activated-Cost Hybrid/Phyrexian Pip Payment

<!-- last_updated: 2026-07-20 -->

**Date**: 2026-07-20
**Reviewer**: primitive-impl-reviewer (Opus)
**Task**: `scutemob-144` · branch `feat/pb-rs2-activated-cost-hybridphyrexian-pip-payment-every-such`
**CR Rules verified via MCP**: 107.4 (incl. 107.4e/107.4f), 119.4 (+119.4a/b), 601.2h, 605.1/605.1a
**Seeds**: OOS-RS-2 (primary), OOS-OS8-1 (subsumed)

**Engine files reviewed**:
`crates/card-types/src/state/game_object.rs`, `crates/card-types/src/state/player.rs`,
`crates/engine/src/rules/abilities.rs`, `crates/engine/src/rules/mana.rs`,
`crates/engine/src/rules/casting.rs`, `crates/engine/src/rules/command.rs`,
`crates/engine/src/rules/protocol.rs`, `crates/engine/src/state/hash.rs`,
`crates/engine/src/testing/replay_harness.rs`, `crates/engine/src/testing/script_schema.rs`,
`crates/simulator/src/legal_actions.rs`

**Test files reviewed**:
`crates/engine/tests/primitives/pb_rs2_activated_pip_payment.rs`,
`crates/engine/tests/core/pb_rs2_hybrid_phyrexian_activation_roster.rs`,
`crates/engine/tests/core/protocol_schema.rs`,
`crates/engine/tests/casting/mana_filter.rs`,
`crates/engine/tests/scripts/harness_equivalence.rs`,
`crates/card-types/src/state/player.rs` (`#[cfg(all(test, debug_assertions))]` module),
`crates/simulator/src/legal_actions.rs` (test module)

**Card defs reviewed (9)**: `birthing_pod`, `graven_cairns`, `twilight_mire`, `sunken_ruins`,
`flooded_grove`, `rugged_prairie`, `fetid_heath`, `cascade_bluffs`, `drivnod_carnage_dominus`

---

## Verdict: **needs-fix** (0 HIGH · 6 MEDIUM · 12 LOW = 18 findings)

> **Count correction (2026-07-20, post-`/review`)**: this header originally read
> "0 HIGH · 5 MEDIUM · 7 LOW" and the fix commit repeated "12 findings", but the tables below
> enumerate **18** rows (6 MEDIUM including the #10/#13 cross-reference, 12 LOW). The fix cycle
> applied all 18 — no work was missing — only the headline was wrong. Corrected here so a future
> reader does not trust the smaller number.

The core of this PB is **correct and well-executed**. I independently re-walked every hop of
the defect chain and every hop of the fix, and the load-bearing claims hold up:

- The flatten now precedes the `mana_value() > 0` gate in **both** handlers, and the Phyrexian
  life deduction is a genuine **sibling** of that gate in both — verified by reading
  `abilities.rs:769-816` and `mana.rs:221-366`, not by trusting the handoff. The exact bug the
  PB exists to fix has **not** been reintroduced anywhere I could find.
- The combined CR 119.4 life check (`ability_cost.life_cost + phyrexian_life`, checked **once,
  before any mutation**) is correct under CR 119.4 + CR 601.2h/602.2b, and I traced every
  `life_total` read/write in `handle_activate_ability` (4 sites) to confirm there is no
  remaining ordering hole. I also independently checked `casting.rs`'s three life-paying sites
  (Phyrexian `:4028`, Bolas's Citadel `:4047`, warp/pitch `:4152`/`:4175`) and confirmed the
  sequential check-then-deduct ordering there is *incidentally* equivalent to a combined check,
  because every later check reads an already-reduced `life_total`. **No circumvention by ordering.**
- One flatten implementation, three call paths, all routed through it (`game_object.rs:182` ←
  `casting.rs:6510` wrapper ← `abilities.rs:771`, `mana.rs:223`, `casting.rs:3991`, plus
  `legal_actions.rs:1041` calling the inherent method directly). AC 5119 satisfied.
- The new CR 107.4e choice validation is correct for **both** pip shapes and rejects no legal
  choice: `ColorColor(a,b)` accepts `Color(a)`/`Color(b)`/`None`, rejects `Generic` and any third
  color; `GenericColor(c)` accepts `Color(c)`/`Generic`/`None`, rejects any other color. Matches
  CR 107.4e verbatim ("paid in one of two ways, as represented by the two halves").
- `HASH_SCHEMA_VERSION` staying 63 is **independently confirmed**: `state/hash.rs` has zero
  `HashInto` arms for the `Command` enum (all 20+ "Command" hits are `ZoneId::Command`,
  `ObjectFilter::Commander`, `LossReason::CommanderDamage`, `ZoneType::Command`, prose). Correct.
- PROTOCOL 26→27 done by the book: version bumped, History line added, `PROTOCOL_HISTORY` row
  **appended** (26 untouched), fingerprint const and tail row in lockstep,
  `protocol_version_sentinel` updated to 27, `FROZEN_HISTORY_PREFIX_DIGEST` re-pinned with a
  PB-RS2 note. No re-pin-without-bump cheat.
- **The "repaired stale tests" in `mana_filter.rs` are an honest repair, not a blessed wrong
  outcome.** The *setup* changed (pool primed with one half of the pip, matching the new
  convention) and the *assertions got strictly stronger* — a new whole-pool `total_net == 1`
  delta assertion was added that would fail loudly under the old free-pip behavior (which
  produced +2). This is the opposite of bending an assertion to match new behavior.
- **No golden scripts under `test-data/` were changed**, and none needed to be: `hybrid_choices`/
  `phyrexian_life_payments` appear in zero script files, and I verified the `tap_for_mana`
  `ability_index` change is behavior-preserving — a corpus-wide grep for `"ability_index": [1-9]`
  returns exactly **one** hit (`baseline/script_189_reconfigure.json:218`), and it is an
  `activate_ability`, not a `tap_for_mana`. The handoff's claim is true.

The findings below are real but none of them make the engine produce wrong game state today.
The two that matter most are a **ratchet gap around the `birthing_pod` Complete flip** (the
only coverage flip in the PB is protected by a test that silently skips itself if the flip is
reverted) and a **now-mis-attributed pre-existing test** in `mana_filter.rs` that passes for the
wrong reason. There is also one **factual inaccuracy in `memory/primitive-wip.md`** that a
future reader would rely on.

### Items the fix cycle MUST execute (this reviewer had no Bash tool)

Per the note in the review brief, the following are assertions I could not verify by reading and
that the fix cycle is required to run and record:

1. `~/.cargo/bin/cargo test -p mtg-engine --test core protocol_schema` — confirm
   `PROTOCOL_SCHEMA_FINGERPRINT` `f035e797…fe3e` and `FROZEN_HISTORY_PREFIX_DIGEST`
   `efadf863…5ebf` are the values the test actually computes. I verified the *procedure* was
   followed but cannot verify the two 64-hex digests by inspection.
2. `~/.cargo/bin/cargo test --workspace` — confirm the claimed all-green, and specifically that
   `crates/card-types`' new `#[cfg(all(test, debug_assertions))]` module actually runs (a
   `card-types` unit-test module is easy to have excluded from the workspace test invocation the
   handoff used).
3. `~/.cargo/bin/cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` **plus**
   `tools/check-defs-fmt.sh` (SR-35 — the 9 touched defs).
4. **The plan's §4 / §12 SR-6 freshness check, which the handoff never records having run**:
   after the `card-types` move, run `~/.cargo/bin/cargo check -p mtg-engine -v` following a
   subsequent *engine-only* edit and confirm `mtg-card-defs` reports `Fresh`. Step 9 of the wip
   claims "all gates green" but this specific checklist item is absent from its list.
5. Confirm the 210 approved golden scripts still pass (`cargo test --test run_all_scripts`) —
   expected green given #0 script edits, but it is the SR-9c gate for the `script_schema.rs`
   field addition and the `tap_for_mana` `ability_index` behavior change.

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `testing/replay_harness.rs:315-330` | **`parse_hybrid_choices` silently drops unparseable entries, shifting every later pip's choice.** Positional misalignment, not just a dropped value. **Fix:** push a sentinel/`Err` instead of `filter_map`-dropping. |
| 2 | MEDIUM | `rules/command.rs:118-131`, `:44-58` | **Doc says "Length must match the hybrid pip count"; nothing enforces it.** Over-long and short vectors are silently tolerated. **Fix:** either enforce the length or correct the doc to describe the actual permissive contract. |
| 3 | LOW | `rules/mana.rs:341-344` | **`.expect()` on the wire/library path.** Provably unreachable, but avoidable. **Fix:** restructure to bind `mana_cost` in the same `if let`. |
| 4 | LOW | `simulator/legal_actions.rs:348-351`, `:482-486` | Two more `.expect("has_pip_cost checked Some")`. **Fix:** `let Some(mc) = … else { … }`. |
| 5 | LOW | `simulator/legal_actions.rs:1008-1022` | **Hybrid-half preference reads only the *pool*, but affordability is then tested with `can_afford` (which consults untapped sources).** Produces false negatives (action not offered though payable via the other half). No illegal or lethal action is ever offered — bot completeness only. **Fix:** try the other half as a second candidate plan before returning `None`. |
| 6 | LOW | `rules/abilities.rs:698-707`, `:786-796` | **The plan's §11.6 out-of-scope item (`abilities.rs`'s missing CR 119.4 check on `life_cost`) was silently taken.** It is now checked in both branches. Correct and welcome — but §11.6 required "take it *and say so in the commit*", and the wip discloses only the *Phyrexian-combined* half. **Fix:** disclose in the wip/commit message. |
| 7 | LOW | `card-types/src/state/player.rs:253-272` | **The guard's doc comment never says `debug_assert!` is a no-op in release.** Plan §6.4 says it explicitly; the code does not. A future reader may treat the guard as load-bearing. **Fix:** add one sentence — release correctness rests on the call sites flattening, not on this assert. |
| 8 | LOW | `rules/abilities.rs`, `rules/mana.rs` | **Call sites still route through `super::casting::flatten_hybrid_phyrexian`.** Plan §4 explicitly said *"Do NOT … call `crate::rules::casting::flatten_hybrid_phyrexian` from `mana.rs` — a layering smell."* AC 5119 (one implementation) is satisfied, but the stated layering goal is not. **Fix:** call the inherent `ManaCost::flatten_hybrid_phyrexian` and map the `String` error locally, as `legal_actions.rs:1041` already does. |
| 9 | LOW | `rules/mana.rs:174-178` vs the new fields | **Asymmetric extraneous-choice handling.** `chosen_color` supplied for a fixed-color ability is rejected as `InvalidCommand`; `hybrid_choices`/`phyrexian_life_payments` supplied for a pip-free cost are silently ignored. **Fix:** mirror the `chosen_color` strictness, or document why not. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 10 | MEDIUM | `birthing_pod.rs` | **The `inert → Complete` flip is not protected by any ratchet.** See Finding 10. |
| 11 | LOW | 7 filter lands | Header line 3 is a single ~190-char comment line in all 7 files. Cosmetic; `rustfmt` does not reflow comments. **Fix:** wrap to the file's prevailing width. |

## Test Findings

| # | Severity | File | Description |
|---|----------|------|-------------|
| 12 | MEDIUM | `tests/casting/mana_filter.rs:193-222` | **`test_filter_land_tap_required` now passes for the wrong reason.** See Finding 12. |
| 13 | MEDIUM | `tests/primitives/pb_rs2_activated_pip_payment.rs:891-904` | **`birthing_pod_activation_charges_the_phyrexian_pip` silently self-skips.** See Finding 10. |
| 14 | MEDIUM | `tests/primitives/pb_rs2_activated_pip_payment.rs:408-456` | **`monocolored_hybrid_payable_as_two_generic` is near-vacuous.** See Finding 14. |
| 15 | LOW | `tests/primitives/pb_rs2_activated_pip_payment.rs:1120-1124` | `residue_guard_test_lives_in_card_types_player_rs` has an **empty body** — it asserts nothing and inflates the "15 tests" count. **Fix:** make it a `//` module comment, not a `#[test]`. |
| 16 | LOW | `tests/primitives/…:756-884` | The filter-land table never proves **`half_b` is payable**. Oracle allows either half; only `half_a` is exercised on the accept path. **Fix:** add a fourth case (pool `half_b`, choice `Color(half_b)` → `Ok`). |
| 17 | LOW | `card-types/src/state/player.rs:509-553` | The residue guard is exercised only through `can_spend`; `spend`'s copy of the guard has no test. **Fix:** add a third `#[should_panic]` calling `spend`. |
| 18 | LOW | `tests/primitives/…:226-256` | The `{R}`-half branch of `hybrid_activated_cost_payable_with_either_half` asserts success but never asserts the pool drained, unlike the `{B}` branch. **Fix:** add the `pool_amount(… Red) == 0` assertion. |

---

## Finding Details

### Finding 1: `parse_hybrid_choices` positional shift on an unparseable entry

**Severity**: MEDIUM
**File**: `crates/engine/src/testing/replay_harness.rs:310-330`
**CR Rule**: 107.4e — each hybrid symbol is an independent pip; the flattener indexes
`hybrid_choices` **by position** (`game_object.rs:210-211`, `hybrid_choices.get(i)`).
**Issue**: The parser uses `filter_map`, so a script writing `["bogus", "red"]` for a two-pip
cost yields `[Color(Red)]` — which the flattener applies to **pip 0**, and pip 1 falls through
to its `None` default. The doc comment defends this as "mirrors `chosen_color`'s permissive
parse", but `chosen_color` is a single scalar where dropping it is positionally harmless; a
positional vector is not. This is a silent-wrong-payment footgun that sits directly against the
grain of the strict CR 107.4e validation this same PB added on the engine side.
**Fix**: Make `parse_hybrid_choices` fail loudly on an unrecognized token — either return
`Option<Vec<_>>`/`Result` and have the arm return `None` (script step rejected), or push a
deliberately-illegal sentinel that the engine's new CR 107.4e validation will reject. Do **not**
keep `filter_map`.

### Finding 2: `Command` doc claims a length invariant that is not enforced

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/command.rs:118-124` (and `:44-51` for `TapForMana`)
**Issue**: Both new fields are documented "Length must match the hybrid pip count after cost
calculation." Nothing checks this. `flatten_hybrid_phyrexian` uses `.get(i)` / `.copied()
.unwrap_or(false)`, so a **short** vector silently defaults and an **over-long** vector is
silently ignored. A client sending `phyrexian_life_payments: [true]` to an ability with no
Phyrexian pip gets a silent no-op rather than an error, even though the same client sending a
bad `chosen_color` gets a clean `InvalidCommand`. Given that this PB's whole thesis is "a
payment channel that silently defaults is how OOS-RS-2 happened", the mismatch matters.
**Fix**: Either (a) enforce `hybrid_choices.len() <= cost.hybrid.len()` and
`phyrexian_life_payments.len() <= cost.phyrexian.len()` in the flattener, returning `Err`, **or**
(b) reword both doc comments to state the actual contract ("may be shorter than the pip count;
missing entries take the documented default. Extra entries are ignored."). (a) is preferred and
is one `if` per field.

### Finding 10: The one coverage flip in this PB has no ratchet

**Severity**: MEDIUM
**Files**: `crates/card-defs/src/defs/birthing_pod.rs:87`,
`crates/engine/tests/primitives/pb_rs2_activated_pip_payment.rs:891-904`,
`crates/engine/tests/core/pb_rs2_hybrid_phyrexian_activation_roster.rs`
**Oracle (MCP-verified)**: "{1}{G/P}, {T}, Sacrifice a creature: Search your library for a
creature card with mana value equal to 1 plus the sacrificed creature's mana value, put that
card onto the battlefield, then shuffle. Activate only as a sorcery."
**Issue — two parts.**

(a) **The card def itself is honest.** I walked it against oracle text: `{3}{G/P}` printed cost
(`:25-29`) ✓; activation cost `Sequence[Mana{generic:1, phyrexian:[Single(Green)]}, Tap,
Sacrifice(creature)]` (`:38-49`) ✓; `max_cmc_amount == min_cmc_amount == Sum(Fixed(1),
ManaValueOfSacrificedCreature)` for "**equal to** 1 plus" (`:60-67`) ✓ — the paired bound is the
correct encoding of "equal to", not a `max`-only approximation; `destination: Battlefield
{tapped:false}` then an explicit `Effect::Shuffle` for "then shuffle" (`:71-77`) ✓ — and the
comment correctly explains why `shuffle_before_placing: false` plus a trailing `Shuffle` is the
right shape; `TimingRestriction::SorcerySpeed` ✓. Zero TODOs, zero stubs. **The flip is honest.**
And the positive paths in the test file (`:943-1029`) genuinely prove it: pay-with-mana succeeds,
pay-with-life succeeds and deducts exactly 2, mana-only-with-`[false]` is rejected. This is not
the `project_legal_but_wrong_gap` failure mode.

(b) **But nothing pins it.** The test that proves all of the above begins with a
`if !def.completeness.is_complete() { eprintln!(…); return; }` self-skip (`:896-904`). If a
future change reverts `birthing_pod` to `inert`/`partial`, this test **silently passes with zero
assertions**, and the roster sweep — which the wip claims performs this check — does **not**: it
only walks `AbilityDefinition::Activated` costs and never reads `completeness` at all. The PB's
sole coverage flip is therefore unratcheted.

**Additionally, the handoff is factually wrong about this.** `memory/primitive-wip.md:215-217`
states: *"Coverage flips: 1 (`birthing_pod`, `inert` → `Complete`). Confirmed, not just estimated
— `all_cards()` + `def.completeness.is_complete()` checked directly in the roster-sweep test."*
No such check exists in `pb_rs2_hybrid_phyrexian_activation_roster.rs`. A future reader will
trust that sentence.

**Fix**: (i) Add an assertion to the roster sweep (or a new test) pinning
`all_cards()`'s "Birthing Pod" entry as `completeness.is_complete()`, with a comment saying it
guards PB-RS2's only flip. (ii) Delete the self-skip from
`birthing_pod_activation_charges_the_phyrexian_pip` and let it fail loudly instead — with (i) in
place, an honest de-flip would already be a loud failure, so the skip has no remaining purpose.
(iii) Correct the wip's yield paragraph to say the completeness assertion is now real (after the
fix) rather than claiming it already was.

### Finding 12: `test_filter_land_tap_required` now passes for the wrong reason

**Severity**: MEDIUM
**File**: `crates/engine/tests/casting/mana_filter.rs:193-222`
**Issue**: The test taps Fetid Heath, activates the filter ability with an **empty pool** and
`hybrid_choices: vec![]`, and asserts only `result.is_err()`. Its doc comment claims it "returns
`PermanentAlreadyTapped`". Post-PB-RS2 it no longer does: in `handle_tap_for_mana` the mana
legality check (`mana.rs:237-244`) runs at step 5b, **before** the tap check at step 6
(`mana.rs:295-297`), so the empty pool produces `InsufficientMana` and the test never reaches
the tapped-permanent path it exists to cover. It is now a duplicate of
`hybrid_pip_in_mana_ability_cost_requires_mana` wearing a "tap required" label. This is exactly
the RS1 standard the brief invoked: reverting the tap check in `mana.rs` would **not** make this
test fail.
**Fix**: Prime the pool with 1 White (as the sibling tests in the same file now do), pass
`hybrid_choices: vec![HybridManaPayment::Color(ManaColor::White)]`, and tighten the assertion to
`matches!(result, Err(GameStateError::PermanentAlreadyTapped(_)))` — restoring what the test
claims to prove. Cite CR 118.3 as it already does.

### Finding 14: `monocolored_hybrid_payable_as_two_generic` cannot detect a wrong amount

**Severity**: MEDIUM
**File**: `crates/engine/tests/primitives/pb_rs2_activated_pip_payment.rs:408-456`
**CR Rule**: 107.4e — "a monocolored hybrid symbol such as {2/B} can be paid with either one
black mana or **two** mana of any type."
**Issue**: The test puts **2** colorless in the pool, pays a `{2/B}` with `[Generic]`, and
asserts only that `process_command` returns `Ok`. It never asserts the pool afterwards. If
`flatten_hybrid_phyrexian`'s `GenericColor` + `Generic` arm were changed from `flat.generic += 2`
(`game_object.rs:240`) to `+= 1`, this test would still pass — undercharging by exactly the
amount CR 107.4e specifies, which is the *same class* of bug this PB exists to eliminate. The
"2" in the rule is the whole point of the case and is untested.
**Fix**: Assert `pool_amount(&state, p(1), ManaColor::Colorless) == 0` after the successful
activation, and add a negative case: **1** colorless in pool with `[Generic]` → `Err(InsufficientMana)`.
Also add the third CR 107.4e arm while there: `[Color(Black)]` with 1 black in pool → `Ok` (the
"either one black mana" half), which is currently only covered by the `None`-default test.

---

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 107.4e (hybrid, two halves) | Yes | Yes | `hybrid_activated_cost_payable_with_either_half`, `hybrid_choice_must_name_a_component_of_the_pip`, `hybrid_empty_choices_defaults_to_first_color`, `filter_land_charges_its_hybrid_pip` |
| 107.4e (monocolored `{2/B}` = 1 colored **or 2** generic) | Yes (`game_object.rs:237-255`) | **Weakly** | Finding 14 — the "2" is not asserted |
| 107.4f (Phyrexian: color **or** 2 life) | Yes | Yes | `phyrexian_activated_cost_payable_with_mana` / `…_with_two_life` |
| 107.4f (hybrid-Phyrexian two-color choice) | **No — documented limitation** | n/a | `game_object.rs:267-277` hard-codes the first color, with an explicit comment and a plan §11.3 pointer. Correctly scoped out: the roster sweep proves no card carries one in an *activation* cost. Acceptable. |
| 119.4 (life ≥ payment) | Yes, all 3 sites | Yes | activate (`abilities.rs:786-796`), tap-for-mana (`mana.rs:252-262`), cast repair (`casting.rs:4020-4027`); `phyrexian_life_payment_requires_sufficient_life` incl. the exactly-2 boundary |
| 119.4 combined total (CR 601.2h/602.2b) | Yes | Yes | `phyrexian_and_explicit_life_cost_check_combined_total` (3 life → Err, 4 life → Ok/0) — this test would fail against the runner's disclosed first-pass implementation, so it is non-vacuous |
| 119.4b (0 life always payable) | Yes | Indirect | Both handlers short-circuit on `combined_life_cost > 0`; `mana.rs:245-247` cites 119.4b explicitly |
| 601.2h (any-order payment) | Yes | Yes | Cited in both handlers' comments as the justification for the combined check |
| 602.2b (activation cost ≡ mana cost) | Yes | Yes | The PB's load-bearing citation; present in `abilities.rs:759-768` |
| 605.1a (mana abilities) | Yes | Yes | `mana.rs` path fixed — the plan's §0.2 correction was implemented, which is what makes the PB actually repair its live-wrong roster |
| 202.3f/202.3g (mana value of pips) | Unchanged | Yes | `phyrexian_paid_with_life_skips_the_mana_gate` pins the raw-mv-1 / flat-mv-0 case |
| 104.3b (bot non-suicide policy) | Yes (simulator) | Yes | `provider_never_offers_a_suicidal_phyrexian_life_plan`, 1/2/5-life boundaries as three distinct outcomes, with an end-to-end `process_command` proof at 5 |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| `birthing_pod` | **Yes** (MCP-verified line by line) | 0 | **Yes** | Flip to `Complete` is honest; unratcheted — Finding 10 |
| `graven_cairns` | Yes | 0 | Input side yes, output side no | `known_wrong` correctly retained; note accurate |
| `twilight_mire` | Yes | 0 | same | note accurate |
| `sunken_ruins` | Yes | 0 | same | note accurate |
| `flooded_grove` | Yes | 0 | same | note accurate |
| `rugged_prairie` | Yes | 0 | same | note accurate |
| `fetid_heath` | Yes | 0 | same | note accurate |
| `cascade_bluffs` | Yes | 0 | same | note accurate |
| `drivnod_carnage_dominus` | n/a (unauthored) | — | — | Note reworded per plan §0.5; stays `partial`. Wording is slightly forward-looking ("actually charged at activation time" describes a cost that isn't authored yet) but is qualified in the same sentence. Acceptable. |

**On the 7 filter lands specifically** — I verified the claim in each note that "the input-side
pip really IS now charged even though the output-side simplification remains." It is: all 7
carry `Cost::Sequence([Cost::Mana{hybrid:[ColorColor(a,b)]}, Cost::Tap])`, all 7 appear in the
pinned roster set, and `filter_land_charges_its_hybrid_pip` drives all 7 through
`Command::TapForMana` with empty-pool → `Err`, correct-half → `Ok` with `total_net == +1`, and
wrong-half-only → `Err`. The `+1` (not `+2`) delta assertion is the specific, non-vacuous proof
that the pip is charged. The surviving `AddManaFilterChoice` fixed-mode blocker is real and
correctly keeps all 7 at `known_wrong` — **zero optimistic flips**, matching plan §8.1 and
`feedback_pb_yield_calibration`.

## Wire / Hash Discipline (SR-8)

| Check | Result |
|---|---|
| `PROTOCOL_VERSION` 26 → 27 | ✓ `protocol.rs:260` |
| History line `- 27:` added, names both variants + CR refs | ✓ `protocol.rs:248-259` |
| `PROTOCOL_HISTORY` row 27 **appended**, row 26 untouched | ✓ `protocol.rs:486-492` |
| Tail row fingerprint == `PROTOCOL_SCHEMA_FINGERPRINT` | ✓ both `f035e797…fe3e` |
| `protocol_version_sentinel` updated to 27 | ✓ `tests/core/protocol_schema.rs:868` |
| `FROZEN_HISTORY_PREFIX_DIGEST` re-pinned with a PB-RS2 note | ✓ `tests/core/protocol_schema.rs:146-149` |
| Digest values verified against actual test output | **UNVERIFIED — fix cycle must run item 1 above** |
| `HASH_SCHEMA_VERSION` stays 63 | ✓ **independently confirmed**: no `HashInto` arm for `Command` anywhere in `state/hash.rs`; no `GameState`-reachable type's shape changed |
| Closure type count unchanged (`HybridManaPayment` already reachable via `CastSpellData`) | ✓ as claimed |

## Deviations from the Plan — assessed

| Plan deviation (as disclosed in wip) | Assessment |
|---|---|
| `activate_ability:hybrid` → `activate_ability:phyrexian` | **Justified.** The roster sweep proves no card has a hybrid pip in a stack-using activated ability, so the plan's label was unbuildable. The substitute exercises the identical `abilities.rs` block. Documented in three places. |
| Reject-path-only equivalence for `activate_ability:phyrexian` | **Acceptable with the mitigation applied.** The explicit `assert_eq!(phyrexian_life_payments, &vec![true])` non-vacuity check at `harness_equivalence.rs:1831-1837` genuinely distinguishes it from the historical `equip` vacuity class (two empty defaults compared equal). The sibling `tap_for_mana:hybrid` scenario **is** a full accept-path proof with a real primed pool (`FILTER_HYBRID_JSON:1402`), so the PB is not relying on reject-only coverage. |
| `InsufficientLife` instead of `InvalidCommand` for the combined check | **Correct call.** Matches the pre-existing structured checks in the same two functions; `InvalidCommand` would have been stringly-typed noise. |
| Self-caught CR 119.4 first-pass bug, disclosed | Good practice. The test that caught it (`phyrexian_and_explicit_life_cost_check_combined_total`) is retained and non-vacuous. |
| §11.6 (`abilities.rs` `life_cost` CR 119.4 check) taken | Correct, but under-disclosed — Finding 6. |
| Plan §4's "don't call it from `casting.rs`" layering directive | **Not followed** — Finding 8 (LOW; AC 5119 still met). |

## Previous Findings

None — this is the first review of PB-RS2.
