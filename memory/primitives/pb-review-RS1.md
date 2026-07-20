# Primitive Batch Review: PB-RS1 — Reconcile Library Top/Bottom (OOS-RS-1)

<!-- last_updated: 2026-07-19 -->

**Date**: 2026-07-19
**Reviewer**: primitive-impl-reviewer (Opus)
**Task**: `scutemob-143` · commits `eaaa0f9d`, `6161ecb3`
**CR Rules verified at source (MCP)**: 121.1, 401.4, 401.7, 701.16 (Investigate), 701.18 (Play),
701.19 (Regenerate), 701.20/701.20a (Reveal), 701.22/a-d (Scry), 701.23 (Search), 701.24 (Shuffle),
701.25/a-d (Surveil), 702.85 (Cascade)

**Engine files reviewed**: `crates/card-types/src/state/zone.rs`,
`crates/card-types/src/cards/card_definition.rs`, `crates/engine/src/effects/mod.rs`,
`crates/engine/src/testing/replay_harness.rs`, `crates/engine/src/rules/replacement.rs`,
`crates/engine/src/rules/events.rs`, `crates/engine/src/rules/resolution.rs`,
`crates/engine/src/rules/casting.rs`, `crates/engine/src/rules/turn_actions.rs`,
`crates/engine/src/rules/copy.rs`
**Test artifacts reviewed**: `tests/mechanics_m_z/library_ordering.rs` (new),
`tests/mechanics_m_z/main.rs`, `tests/mechanics_m_z/reveal_and_route.rs`,
`tests/core/pb_rs1_roster_sweep.rs` (new), `tests/core/main.rs`,
`tests/primitives/pb_os8_look_at_top_then_place.rs`, `tests/scripts/harness_equivalence.rs`
**Golden scripts reviewed**: `etb-triggers/002`, `stack/012`, `stack/018`, `stack/034`, `stack/199`
**Card defs reviewed**: 0 changed (verified). Spot-checked for the roster-delta claim:
`experimental_augury`, `senseis_divining_top`, `muxus_goblin_grandee`, `aven_mindcensor`

> **Tooling note**: this review session had no Bash tool. `git diff main...HEAD` could not be
> executed; every claim below is verified by reading the worktree's current source and by
> exhaustive Grep for stragglers. Where a claim depends on *what changed* rather than *what is*
> (specifically: golden-script assertion integrity), I verified via the per-script
> `generation_notes` the runner wrote plus semantic consistency of the surviving assertions.
> That is strong but not diff-equivalent evidence; see "Residual verification gap" at the end.

---

## Verdict: needs-fix

The core of this PB is **correct and well-executed**. The CR decision is right and I re-derived it
independently: CR 121.1 defines drawing as taking the *top card*, `draw_card` takes `Zone::top()` =
`v.last()`, therefore top = last element and bottom = index 0 — camp A. CR 702.85a (cascade →
bottom, implemented as `push_front`) and CR 401.7 corroborate. Camp B had no CR support in either
direction. `Zone::top_n` is a single shared helper that agrees with `top()`; **all four** read arms
use it (Surveil was **not** dropped); zero open-coded `object_ids().take(n)` survive; the Scry
bottom-write is routed through `expect_move_object_to_bottom_of_zone`; the harness comment and loop
now agree; the probe genuinely discriminates (pre-fix failure output is real and the assertions are
position/identity-based, not membership); the de-vacuoused `reveal_and_route.rs` tests use 5-card
libraries against `count: 4` with bottom decoys and ObjectId-identity assertions; the roster sweep
walks the full serde tree rather than a shallow match; PROTOCOL is still 26 and HASH still 63; and
there are **zero card-def diffs**. Every CR citation the runner claims to have fixed is correct
against the MCP.

Findings are **1 MEDIUM correctness, 1 MEDIUM citation, 4 LOW**. None invalidates the PB; the
MEDIUM correctness item is a *fifth* inverted top-N library read (`RestrictSearchTopN`, live on
`aven_mindcensor` which is `Completeness::Complete`) that the spec's four-arm inventory missed and
that is now inverted against the newly-corrected `top_n` — precisely the failure mode the plan
warned about for Surveil.

---

## Arbitration calls I was asked to make

### §5c Option 1 (narrow local `LibraryPosition::Bottom` dispatch) — **APPROVED**

| Test | Verdict | Evidence |
| --- | --- | --- |
| (a) Correct behavior? | **Yes** | `effects/mod.rs:5032-5048` (RevealAndRoute `unmatched_dest`) and `:5174-5197` (LookAtTopThenPlace `rest_to`) branch to `expect_move_object_to_bottom_of_zone` iff the destination is `ZoneTarget::Library { position: LibraryPosition::Bottom, .. }`. Under CR 121.1 the bottom is `push_front`; correct. `new_id` handling and `zone_move_event` emission are preserved on **both** branches. |
| (b) Respects the exclusion? | **Yes** | `resolve_zone_target` at `:7856` is **untouched** — still `ZoneTarget::Library { owner, .. }`, still discarding `position`. No type added, no field added, `LibraryPosition`'s general zero-read status unchanged. |
| (c) Half-state worse than before? | **No** | The `Top` case is unaffected and remains correct by construction (`expect_move_object_to_zone` = `push_back` = the top end). Only the previously-broken `Bottom` case changes, and only in the two arms. Nothing regressed. |

The plan's Finding B is confirmed against source: the spec's "the siblings share the shape" premise
is **false**. Actual breakdown is 1 direct site (Scry), 1 non-applicable (Surveil writes to the
graveyard per CR 701.25a — correctly left alone; `effects/mod.rs:3129` still moves to
`ZoneId::Graveyard`, nothing was wrongly "fixed" there), and 2 indirect sites gated on the
out-of-scope `LibraryPosition` discard.

### Scry's all-N-to-bottom fallback — **correctly NOT changed**

CR 701.22a permits putting "any number" on the bottom; all-N is a legal (weak) deterministic
choice. `effects/mod.rs:3089-3098` leaves it. Correct restraint.

### `034_brainstorm_then_fetch.json` residual drift — **judgment confirmed**

I read every `note` in the body (`:125`–`:255`). `:158` was updated to the CR-correct draw order;
`:166` ("Hand: 1 card (Counterspell)") is consistent with the `generation_notes` rationale;
`:180`/`:211`/`:224` describe shuffle-randomized state and are order-insensitive. I found **no
material residual drift** that encodes the old inversion. Out-of-scope call is sound.

---

## Engine Change Findings

| # | Severity | File:Line | Description |
| --- | --- | --- | --- |
| 1 | **MEDIUM** | `crates/engine/src/effects/mod.rs:3010-3022` | **A fifth top-N library read survives, still inverted.** `RestrictSearchTopN` computes "the top N" as the N *lowest ObjectIds*, not `Zone::top_n`. Live-wrong on `aven_mindcensor` (`Completeness::Complete`). **Fix:** route through `Zone::top_n(n)`, or file as a seed if deferred. |
| 2 | **MEDIUM** | `crates/engine/src/rules/events.rs:766` | **Wrong Scry CR citation survives the PB's own citation sweep.** `GameEvent::Scried` still cites CR 701.18 (= Play). Plan §0.2 mandated *every* Scry reference read CR 701.22. **Fix:** `CR 701.18` → `CR 701.22`. |
| 3 | LOW | `crates/engine/src/rules/replacement.rs:340` | **Wrong Search citation.** "CR 701.19/614.1: Library search replacement matching" — 701.19 is Regenerate; Search is 701.23. Sibling at `:2976` is already correct. **Fix:** `701.19` → `701.23`. |
| 4 | LOW | `crates/engine/tests/mechanics_m_z/reveal_and_route.rs:1` | **The PB fixed this exact citation elsewhere but not here.** Module doc still says "Tests for Effect::RevealAndRoute (CR 701.16a)"; 701.16a is Investigate. **Fix:** `CR 701.16a` → `CR 701.20a`. |
| 5 | LOW | `crates/engine/src/effects/mod.rs:5012` and `:5154` | **Bottom dispatch is asymmetric.** `matched_dest` (RevealAndRoute) and `destination` (LookAtTopThenPlace) get no `LibraryPosition::Bottom` branch. Currently latent — no card def uses them with `Library`. **Fix:** add the same `matches!` branch, or add a comment recording the asymmetry. |
| 6 | LOW | `crates/engine/tests/mechanics_m_z/library_ordering.rs:533-535` | **Comment rationale is inverted.** Says "Top1 and Top2 (declared first, so lower ObjectIds)" — they are declared **last** and carry the **highest** ObjectIds. Assertion is correct; the reasoning is not. **Fix:** correct the comment. |

## Card Definition Findings

None. `git diff main...HEAD -- crates/card-defs/` is empty (verified indirectly: zero `PB-RS1`
occurrences anywhere under `crates/card-defs/`, and the roster sweep is a read-only enumeration).
`crates/card-types/src/cards/card_definition.rs` was touched at `:1658` and `:2020-2022` — both are
**doc comments only**; the `Effect::Scry` and `Effect::RevealAndRoute` variant shapes are unchanged.

---

### Finding Details

#### Finding 1: A fifth top-N library read survives, still inverted (Aven Mindcensor)

**Severity**: MEDIUM
**File**: `crates/engine/src/effects/mod.rs:3010-3022`
**CR Rule**: 121.1 ("A player draws a card by putting the top card of their library into their
hand"); CR 701.23 (Search); CR 614.1
**Oracle** (`aven_mindcensor`): "If an opponent would search a library, that player searches the
top four cards of that library instead."

**Issue**: The `SearchLibrary` arm implements the `RestrictSearchTopN` replacement like this:

```rust
// If search is restricted to top N, sort by library position
// and truncate. Library order is by ObjectId ascending (deterministic).
if let Some(top_n) = search_restriction {
    let mut all_lib: Vec<ObjectId> = /* every object whose zone == lib_id */;
    all_lib.sort();
    let top_ids: HashSet<ObjectId> = all_lib.into_iter().take(top_n as usize).collect();
```

This is the same open-coded-top-N defect PB-RS1 exists to eliminate, in a fifth location the
spec's four-arm inventory never enumerated. It is wrong twice over:

1. **It uses ObjectId order as a proxy for library position.** The comment "Library order is by
   ObjectId ascending" is false as a general claim — after any `Effect::Shuffle`
   (`effects/mod.rs:3139`), any cascade bottoming (`resolution.rs:6014`), or any Scry-to-bottom
   (now `effects/mod.rs:3097`), ObjectId order and `Zone::Ordered` index order are fully decoupled.
   The authoritative position source is the zone vector, which is exactly what `Zone::top_n` reads.
2. **Even where ObjectId order does track insertion order, ascending picks the wrong end.** Under
   the harness's now-correct convention (`replay_harness.rs:213`, `.iter().rev()`), the
   last-declared (bottom) card is pushed first and gets the lowest ObjectId. So
   `all_lib.sort(); .take(4)` selects the **bottom** four.

**Concrete failure scenario**: P1 controls Aven Mindcensor. P2 activates Evolving Wilds and
searches for a basic land. Their library is `[Swamp(bottom), …, Forest, Plains(top)]`. Oracle: P2
may only find a land among the **top four**. The engine instead restricts P2 to the **bottom four**
— P2 finds a Swamp that oracle says is unreachable, or fails to find a Plains that oracle says is
findable. `aven_mindcensor.rs` takes `..Default::default()`, and `CardDefinition::default()` sets
`completeness: Completeness::Complete` (`card_definition.rs:268`) — so this is a live-wrong
`Complete` card, the same Invariant #9 class that motivated this PB.

**Standing**: pre-existing, not introduced here, and outside the spec's literal four-arm scope — the
PB is a strict improvement without it. But post-PB this site is now inverted *against* the corrected
`Zone::top_n`, which is precisely the condition the plan called "strictly worse than today's uniform
wrongness" when arguing Surveil must not be dropped. The same argument applies here.

**Fix**: replace the ObjectId-proxy block with a positional read —
`let top_ids: HashSet<ObjectId> = state.zones.get(&lib_id).map(|z| z.top_n(*top_n as usize)).unwrap_or_default().into_iter().collect();`
— delete the false "Library order is by ObjectId ascending" comment, cite CR 701.23/121.1, and add
a discriminating test (library longer than 4, the matching land placed at the true bottom, assert it
is **not** findable). If deferred instead, file it as a new seed **OOS-RS1-2** in
`memory/primitives/rider-seed-triage-2026-07-19.md` §1c with class **correctness** and note that
`aven_mindcensor` is `Complete`.

#### Finding 2: `GameEvent::Scried` still cites CR 701.18 (Play)

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/events.rs:766`
**CR Rule**: 701.18 is **Play**; 701.22 is **Scry** (both verified via MCP)
**Issue**: Plan §0.2 is explicit: "Every CR 701.19 / 701.18 reference to Scry in this PB's comments
must read CR 701.22." The runner corrected `effects/mod.rs:3075` and `card_definition.rs:1658` but
missed the third occurrence, on the `Scried` event itself. Since the PB's own close-out claims the
Scry citation drift is resolved, a future reader who greps `701.18` will find a surviving Scry
reference and reasonably conclude 701.18 is Scry — reintroducing the drift this PB set out to kill.
**Fix**: `events.rs:766` — `(CR 701.18)` → `(CR 701.22)`.

---

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
| --- | --- | --- | --- |
| 121.1 (draw = top card) | Yes — decisional anchor | Yes | `test_probe_draw_and_scry_agree_on_top`, `test_scry_two_to_bottom_lands_below_everything` (draw_card cross-check) |
| 701.22a (Scry: look at top N, any number to bottom) | Yes — `top_n` read + `push_front` write | Yes | `test_probe_draw_and_scry_agree_on_top`, `test_scry_two_to_bottom_lands_below_everything` |
| 701.22b (scry 0 → no event) | Unchanged by this PB | Not newly tested | `top_n(0)` returns empty; behavior preserved. Note Scry (unlike Surveil `:3112`) has no `n == 0` early-continue, so `Scried { count: 0 }` still emits — **pre-existing**, correctly untouched |
| 701.25a (Surveil: top N → graveyard) | Yes — `top_n` read; graveyard write correctly unchanged | Yes | `test_probe_surveil_mills_the_drawn_card` |
| 701.25c (surveil 0 → no event) | Yes (pre-existing `:3112`) | Not newly tested | Preserved |
| 701.20a (Reveal) | Citation corrected on `RevealAndRoute` | Yes | `reveal_and_route.rs` tests now cite it (except the stale module doc, Finding 4) |
| 702.85a (cascade → bottom) | Pre-existing `push_front`; now reconciled with the read side | Yes | `test_cascade_bottomed_card_is_not_seen_by_next_scry` (real cascade path via `CastSpell`) |
| 401.4 (owner arranges simultaneous inserts) | Yes — arbitrary deterministic order is legal | Implicitly | Scry bottoms in reverse-ObjectId order; legal |
| 401.7 (Nth-from-top on a short library) | Yes — `top_n` saturates | Yes | `test_top_n_over_length_saturates` |
| 701.23 (Search) | **Partially — see Finding 1** | No | `RestrictSearchTopN` reads the wrong end |
| 400.7 (new object on zone change) | Yes — used as the discriminator | Yes | Test 3 and `pb_os8` truncation test both assert ObjectId identity survival |

---

## Item-by-item verification against the review brief

| # | Item | Verdict |
| --- | --- | --- |
| 1 | CR decision (camp A) | **CONFIRMED INDEPENDENTLY.** CR 121.1 is definitional and makes `draw_card` the anchor; `top()` = `v.last()` therefore top = last. CR 702.85a's bottom = `push_front` = index 0 is self-consistent with it. Camp B satisfies neither. CR does **not** point the other way. (CR 103.2 is pre-game setup and inapposite — the plan is right to say so.) |
| 2 | Citation hygiene | **All claimed fixes verified correct via MCP**: 701.19 = Regenerate ✓, 701.18 = Play ✓, 701.22 = Scry ✓, 701.16 = Investigate ✓, 701.20a = Reveal ✓, 701.23 = Search ✓, 701.25 = Surveil ✓. No "fix" is itself wrong. **Three residual misses**: Findings 2, 3, 4. 034 out-of-scope call confirmed sound. |
| 3 | `Zone::top_n` | **PASS.** One shared helper at `zone.rs:175-180`, immediately after `top()`. `v.iter().rev().take(n)` → index 0 is topmost. `n > len` saturates via `take`, no panic. Unordered → empty, matching `top()`. Four contract unit tests at `:204-233`. **Grep for stragglers: zero surviving `object_ids().take(n)` in `crates/`** — the only textual matches are explanatory comments in `library_ordering.rs`. All four arms use it: `:3087`, `:3121`, `:4986`, `:5080`. |
| 4 | Surveil not dropped | **PASS.** `effects/mod.rs:3118-3122` reads via `z.top_n(n)`. Not fixed 3-of-4. |
| 5 | Bottom-writes + §5c scope call | **PASS / APPROVED.** See "Arbitration calls" above. Surveil's graveyard write untouched; Scry's all-N fallback untouched; `resolve_zone_target` untouched. |
| 6 | The probe is real | **PASS.** 5 tests, all position/identity-based. `test_probe_reveal_and_route_sees_the_drawn_card` asserts the hand card is `"Card Gamma"` (revert → `"Card Alpha"`, fails). `test_probe_surveil_mills_the_drawn_card` asserts Gamma in GY **and Alpha not** (revert → fails both). Test 3 uses ObjectId-identity survival (CR 400.7) across a **real cascade** driven by `CastSpell` + `PassPriority` — genuinely cross-subsystem. Test 4's `draw_card` cross-check at `:558-572` is the designed guard against a compensating double-inversion. None is vacuous. Pre-fix failure output in `pb-plan-RS1.md` §16 is consistent with the implemented assertions. **`mod library_ordering;` present at `mechanics_m_z/main.rs:8`** (SR-9a satisfied); `mod pb_rs1_roster_sweep;` at `core/main.rs:27`. |
| 7 | De-vacuoused `reveal_and_route.rs` | **PASS.** Libraries grown to 5 vs `count: 4`, with a deliberate *bottom decoy* whose subtype is chosen to flip the result under a read inversion (e.g. `test_reveal_and_route_none_match` puts a **Goblin** at the true bottom and Elves in the window, so an inverted read lands the Goblin in hand). Assertions are by name and by `assert_eq!(lib_ids, vec![decoy_id])` — ObjectId identity, not membership. CR 121.1 cited at `:41`, `:132`, `:156`, `:236`, `:261`, `:341`, `:390`, `:459`. |
| 8 | 5 scripts + 3 fixtures | **PASS, with the tooling caveat.** Exactly 5 scripts carry `PB-RS1` notes — matching the report, no undisclosed edits. Each `generation_notes` documents a **setup reordering** with an explicit rationale for why the original assertions are preserved (e.g. `stack/012`: "Plains is declared first … to keep Forest winning the first (battlefield) search and Plains the second (hand) search, **matching this script's assertions**"). `harness_equivalence.rs:1106-1112` and `:1266-1267` reverse only `.object()` push order; `DELVE_MOVES` and the expectation tables are untouched. `pb_os8_look_at_top_then_place.rs:538-599` reverses push order and rewrites the doc comment; the assertions (in-window placed; out-of-window **untouched under its ORIGINAL ObjectId**, CR 400.7) are unchanged and remain the strong form — the reversal is a genuine correction of a setup that encoded the old convention, not a papering-over. **No assertion weakening found.** |
| 9 | Replay harness | **PASS.** `replay_harness.rs:207-217`: comment now says scripts declare TOP-TO-BOTTOM and the loop must insert in reverse; the loop is `for card in lib_cards.iter().rev()`. Comment and code agree. `sorted_zone_entries` still governs cross-player order, so SR-9b determinism holds. |
| 10 | No wire bump | **PASS.** `rules/protocol.rs:248` = 26; `state/hash.rs:578` = 63. No `Effect`/`Command`/`GameEvent`/DSL type added or reshaped; `Effect::Scry`/`Surveil`/`RevealAndRoute`/`LookAtTopThenPlace` shapes unchanged. `top_n` is a method on `Zone`, outside the schema closure. No schema fingerprint re-pin — the STOP condition was **not** triggered. |
| 11 | No card-def diffs | **PASS.** Zero `PB-RS1` occurrences under `crates/card-defs/`. `card_definition.rs` edits at `:1658` and `:2020-2022` are comment-only. |
| 12 | Roster count | **PASS.** `tests/core/pb_rs1_roster_sweep.rs` enumerates `all_cards()` and walks `serde_json::to_value(&def)` recursively — reaches every nesting site (`ForEach`, `Conditional`, modes, triggered/activated abilities) by construction, so it is **not** a shallow scan. `contains_key` matches object *keys* only, so effect names appearing inside `completeness` note **strings** correctly do not false-positive. **Spot-checked 3 of the 6 claimed grep over-counts**: `experimental_augury` (mentions `Effect::RevealAndRoute` only in a `known_wrong` note), `senseis_divining_top` (mentions `Effect::Scry` only in an `inert` note + TODOs), `muxus_goblin_grandee` (mentions both only in a `TODO(OOS-OS8-2)` and a `partial` note). All three genuinely do **not** use the primitive. Delta explanation is sound. Minor: the test asserts a floor of `>= 30` rather than the measured 41, so a real 41→31 regression would pass silently — acceptable anti-flake tradeoff, noted not filed. |
| 13 | OOS-RS1-1 filed not fixed | **PASS.** Filed in `rider-seed-triage-2026-07-19.md` §1c line 87, class **capability**, with the corrected line reference (`:7856`, was `:7830`). `resolve_zone_target` is verifiably unfixed (`:7856` still `ZoneTarget::Library { owner, .. }`). The note explicitly records: "Muxus's 'rest on the bottom in a random order' remains inexpressible — **OOS-OS8-2 (muxus authoring) stays gated even after PB-RS1**." Exactly as required. |

---

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
| --- | --- | --- | --- | --- |
| *(none changed)* | n/a | n/a | n/a | Zero card-def diffs — correct per plan §7 |
| `aven_mindcensor` (not changed; found during straggler sweep) | Yes (def matches oracle) | 0 | **No** | `Complete`, but the engine restricts to the **bottom** 4 — Finding 1. Behavior defect, not a def defect |
| `experimental_augury` / `senseis_divining_top` / `muxus_goblin_grandee` (spot-checks) | n/a | pre-existing | n/a | Confirmed: name mentions are TODO/note text only, not `Effect` usage — validates the 47→41 delta |

---

## Residual verification gap (disclosed)

Without a Bash tool I could not run `git diff main...HEAD`, `cargo test --all`, or
`SCRIPT_FILTER=… cargo test --test run_all_scripts`. Consequently:

- **Golden-script assertion integrity** is verified by reading the surviving assertions and the
  runner's per-script `generation_notes` rationale, and by confirming that exactly the 5 reported
  scripts carry `PB-RS1` markers (no undisclosed edits). Every note describes a *setup* reordering
  and explicitly states the assertions were preserved; the surviving assertions are semantically
  consistent with those rationales. I found no evidence of weakening. A one-line
  `git diff main...HEAD -- test-data/ | grep '^[-+].*expected'` during the fix cycle would close
  this to certainty and is **recommended**.
- The gate results in `memory/primitive-wip.md` step 9 (tests/clippy/fmt/`check-defs-fmt.sh`/
  `build --workspace`) are taken as reported, not re-executed.

---

## Recommendation

**Needs a short fix cycle.** The PB's substance is sound and the load-bearing convention change is
correct, complete across the four arms, well-tested, and wire-neutral — this is good work. Required
before close:

1. **Finding 1** — fix `RestrictSearchTopN` to read via `Zone::top_n` (~4 lines + one test), **or**
   file it as **OOS-RS1-2** (class correctness, `aven_mindcensor` is `Complete`). Do not leave it
   silently inverted against the helper this PB just created.
2. **Finding 2** — `events.rs:766` `701.18` → `701.22`.
3. **Findings 3–6** — LOW, batch them into the same commit: `replacement.rs:340` `701.19` → `701.23`;
   `reveal_and_route.rs:1` `701.16a` → `701.20a`; a comment on the `matched_dest`/`destination`
   bottom-dispatch asymmetry; correct the inverted ObjectId rationale at
   `library_ordering.rs:533-535`.
4. **Recommended, not required** — run the `test-data/` diff grep described above to convert the
   script-integrity finding from "strong evidence" to "verified."

No re-plan is warranted. No wire bump. No scope change.
