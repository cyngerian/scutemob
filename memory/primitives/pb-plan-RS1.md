# Primitive Batch Plan: PB-RS1 — Reconcile Library Top/Bottom (OOS-RS-1)

<!-- last_updated: 2026-07-19 -->

**Generated**: 2026-07-19
**Task**: `scutemob-143`
**Branch**: `feat/pb-rs1-reconcile-library-topbottom-revealscry-family-reads-t`
**Primitive**: `Zone::top_n(n) -> Vec<ObjectId>` — one shared top-N accessor that agrees with
`Zone::top()`, replacing four drifted open-coded `object_ids().take(n)` reads; plus routing the
matching "to the bottom" writes through `move_object_to_bottom_of_zone`.
**CR Rules**: 121.1, 401.1–401.7, **701.22 (Scry)**, 701.25 (Surveil), 702.85a (Cascade)
**Class**: CORRECTNESS (Invariant #9 — live-wrong on shipped `Complete` defs)
**Cards affected**: 47 distinct files (behavior repairs, **0 marker flips** — see §7)
**Dependencies**: none
**Deferred items from prior PBs**: none consumed. **Files** one new seed (§9).
**Wire expectation**: **NONE** — no PROTOCOL / HASH bump. Verified in §8.

---

## 0. Verification preamble — what I confirmed at source, and where the spec is WRONG

Per `feedback_verify_cr_before_implement.md`, I treated the brief as a hypothesis, not authority.
Every claim below was read at source in this worktree.

### 0.1 Confirmed exactly as briefed

| Claim | Source | Verdict |
| --- | --- | --- |
| `Zone::top()` = `v.last()` | `crates/card-types/src/state/zone.rs:159-164` | **CONFIRMED** |
| `Zone::object_ids()` returns front-to-back | `zone.rs:130-135` (`v.iter().copied()`) | **CONFIRMED** |
| `draw_card` uses `top()` | `crates/engine/src/rules/turn_actions.rs:1193-1195` | **CONFIRMED** |
| `move_object_to_bottom_of_zone` = `push_front`, doc'd "front (= bottom)" | `crates/engine/src/state/mod.rs:1787-1792` | **CONFIRMED** |
| `push_front` doc cites cascade/CR 702.85a | `zone.rs:165-178` | **CONFIRMED** |
| Four byte-identical `object_ids().take(n)` reads, all commented "ordered from top" | Scry `effects/mod.rs:3082-3090`; Surveil `:3117-3125`; RevealAndRoute `:4985-4993`; LookAtTopThenPlace `:5065-5073` | **CONFIRMED** (all four) |
| Scry's bottom-write uses `expect_move_object_to_zone` (= `push_back` = camp-A TOP) | `effects/mod.rs:3097` | **CONFIRMED** |
| Harness declares "top-to-bottom" then inserts via `builder.object(...)` (append) | `testing/replay_harness.rs:207-212` | **CONFIRMED** |
| `resolve_zone_target` discards `ZoneTarget::Library { position }` | `effects/mod.rs:7830` (`ZoneTarget::Library { owner, .. }`) | **CONFIRMED** |
| `reveal_and_route.rs` tests are vacuous (2-card lib, `count: 4`, membership-only) | `tests/mechanics_m_z/reveal_and_route.rs:65-104` | **CONFIRMED** |

**The core defect is real and reproduces by inspection**: `object_ids().take(n)` returns indices
`0..n`, which is precisely the end `push_front` calls the *bottom* and `top()` never touches.
Scry additionally writes back with `push_back` — **wrong at both ends**, exactly as briefed.

### 0.2 ⚠️ FINDING A — the spec's CR citation for Scry is WRONG (and so is the source comment)

The brief and `rider-seed-triage-2026-07-19.md` §5 both cite **"CR 701.19 (Scry)"**. Verified via
MCP: **CR 701.19 is Regenerate.** Scry is **CR 701.22**. Separately, the in-source comment at
`effects/mod.rs:3075` cites **"CR 701.18"** — CR 701.18 is *Play*. Both citations are wrong, and
the wrong one is propagated into `test-data/generated-scripts/baseline/009_read_the_bones_scry_draw.json:26`.

**Every CR 701.19 / 701.18 reference to Scry in this PB's test citations and comments must read
CR 701.22.** Surveil's in-source `CR 701.25` citation (`effects/mod.rs:3105`) **is correct**.

### 0.3 ⚠️ FINDING B — "the siblings share the shape" is FALSE (scope-material)

The spec §5 scope item 2 says the four arms share Scry's bottom-write shape and asks for all four to
be rerouted. **I checked each. They do not.** Actual shapes:

| Arm | Has a "to the bottom" write? | Mechanism |
| --- | --- | --- |
| **Scry** `:3094-3098` | **YES, hardcoded** | `expect_move_object_to_zone(id, lib_zone)` → `push_back` → camp-A TOP. **Directly fixable.** |
| **Surveil** `:3131-3133` | **NO** | Writes to `ZoneId::Graveyard(p)` (CR 701.25a). **There is no bottom-write. Nothing to reroute.** |
| **RevealAndRoute** `:5033-5039` | **Indirect** | `resolve_zone_target(unmatched_dest, ..)`; card defs pass `LibraryPosition::Bottom`, which `:7830` **discards**. |
| **LookAtTopThenPlace** `:5166-5171` | **Indirect** | `resolve_zone_target(rest_to, ..)`; same discard. Comment at `:5158` even says "Bottom the rest (CR 401)" — it does not. |

So the write-side fix is **1 direct site, 1 non-applicable site, and 2 sites gated on the
explicitly-out-of-scope `LibraryPosition` discard.** This is a genuine collision between the spec's
scope item 2 and its own "NOT in scope" list. Resolution in §5c — I recommend a **narrow,
local** dispatch that does **not** touch `resolve_zone_target` and does **not** widen the PB.

### 0.4 Not a spec error, but worth recording

`Effect::Scry`'s deterministic fallback puts **all** N looked-at cards on the bottom. CR 701.22a
permits "any number," so all-N is a legal (if weak) deterministic choice. **Not in scope to change.**
Do not "fix" it.

---

## 1. CR RULE TEXT (verbatim, MCP-sourced)

**CR 121.1** — *the decisive rule*:
> A player draws a card by putting the top card of their library into their hand. This is done as a
> turn-based action during each player's draw step. It may also be done as part of a cost or effect
> of a spell or ability.

**CR 701.22a** (Scry):
> To "scry N" means to look at the top N cards of your library, then put any number of them on the
> bottom of your library in any order and the rest on top of your library in any order.

**CR 701.22b**: > If a player is instructed to scry 0, no scry event occurs. Abilities that trigger whenever a player scries won't trigger.

**CR 701.25a** (Surveil):
> To "surveil N" means to look at the top N cards of your library, then put any number of them into
> your graveyard and the rest on top of your library in any order.

**CR 702.85a** (Cascade, abridged):
> "Cascade" means "When you cast this spell, exile cards from the top of your library until you
> exile a nonland card whose mana value is less than this spell's mana value. … Then put all cards
> exiled this way that weren't cast **on the bottom of your library** in a random order."

**CR 401.1**: > When a game begins, each player's deck becomes their library.
**CR 401.4**: > If an effect puts two or more cards in a specific position in a library at the same time, the owner of those cards may arrange them in any order. That library's owner doesn't reveal the order in which the cards go into the library.
**CR 401.7**: > If an effect causes a player to put a card into a library "Nth from the top," and that library has fewer than N cards in it, the player puts that card on the bottom of that library.

*(CR 103.2 was checked and is **not** relevant — it covers pre-game companion/commander/sticker
setup, not library ordering. The brief's citation of it is harmless but inapposite. CR 401.2 —
"players can't look at or change the order of cards in a library" — is the general prohibition that
Scry/Surveil are explicit exceptions to.)*

---

## 2. THE CR DECISION — **CAMP A IS AUTHORITATIVE. FIX CAMP B.**

### The argument

**CR 121.1 is definitional, and it makes `draw_card` the anchor.** The rules never define "top of
library" as an abstract coordinate; they define *drawing* as "putting the top card of their library
into their hand." The top of the library is therefore **operationally identified as the card a draw
takes**. Any engine site that disagrees with the draw path about which card is on top is, by CR
121.1, wrong — regardless of which site is more numerous or more convenient.

`draw_card` (`turn_actions.rs:1195`) takes `library.top()`, and `Zone::top()` (`zone.rs:159-164`) is
`v.last()`. **Therefore, by CR 121.1, the topmost card of a library in this engine is the LAST
element of the `Zone::Ordered` vector.** Camp A is not merely load-bearing; it is the camp that
implements the definitional rule.

**Corroboration from CR 702.85a.** Cascade must put non-cast exiles "on the bottom." The engine does
this with `push_front` (`state/mod.rs:1792`, `zone.rs:171-178`), i.e. index 0. Bottom = index 0 and
top = last are the same convention, self-consistently. Camp A satisfies CR 121.1 **and** CR 702.85a
simultaneously with one orientation.

**Camp B satisfies neither.** `object_ids().take(n)` reads indices `0..n`. Under the orientation CR
121.1 forces, those are the *bottommost* n cards. So:
- **Scry (CR 701.22a)** looks at the bottom N instead of the top N, then `push_back`s them onto the
  **top** — a card the player was told to bottom lands where the next draw takes it. Wrong at both ends.
- **Surveil (CR 701.25a)** mills from the bottom of the library.
- **RevealAndRoute / LookAtTopThenPlace** ("look at the top N") read the bottom N.
- A **cascade** that correctly bottoms a card places it exactly where the next **Scry** reads first —
  the two camps actively feed each other garbage.

**Could camp B win instead?** Only by redefining `top()` to `v.first()`, which would require
simultaneously rewriting `draw_card`, `move_object_to_bottom_of_zone`/`push_front`, cascade
(`rules/resolution.rs:5935`, `:6014`), hideaway, and `rules/copy.rs:484-495` — i.e. inverting the
implementation of the one rule (CR 121.1) that *defines* the term, in order to preserve four
comments. **There is no CR support for that direction.** The rules do not privilege index 0.

### DECISION

> **Camp A is authoritative: `top` = last element, `bottom` = index 0.**
> **Citations: CR 121.1 (definitional), corroborated by CR 702.85a and CR 401.7.**
> **All four camp-B reads and the Scry write are the defect. Fix camp B.**

This agrees with the spec's stated expectation, but was derived from CR 121.1 independently and is
**not** contingent on camp A being cheaper to keep. **Nothing here contradicts the spec's premise;
the PB proceeds.** (Per the brief's stop-condition: no stop is warranted.)

**Step 0 remains mandatory anyway** — the CR argument settles *direction*; the probe supplies
*executing evidence* that the defect is live and that the fix closes it.

---

## 3. Step 0 — the discriminating probe (WRITE FIRST, BEFORE ANY EDIT)

**File**: `crates/engine/tests/mechanics_m_z/library_ordering.rs` (**NEW**)
**Register it**: add `mod library_ordering;` to `crates/engine/tests/mechanics_m_z/main.rs`, in
alphabetical position **between `mod ninjutsu;` (`:21`) and `mod madness;`… — precisely: after
`mod ninjutsu;` is wrong; insert between `mod jump_start`-class names. Concretely: place it after
`mod ...` such that alphabetical order holds — insert `mod library_ordering;` **immediately before
`mod madness;` at `main.rs:8`.**

> **SR-9a**: this is a module inside an existing grouped target. **NEVER** create a top-level
> `crates/engine/tests/*.rs` — `tests/no_stray_test_binaries.rs` fails the suite if one appears.
> A missing `mod` line silently deletes the coverage; the gate catches the stray file, **not** the
> missing `mod`. Add the `mod` line in the same edit as the file.

### Setup (shared helper in that file)

Build a **3-card library** for `p(1)` via `GameStateBuilder`, declaring in this order:
`"Card Alpha"`, `"Card Beta"`, `"Card Gamma"`.

`GameStateBuilder::object(...)` appends, so the resulting `Zone::Ordered` vector is
`[Alpha, Beta, Gamma]`. Under the decision in §2, **`Gamma` is the top card** (last element) and
`Alpha` is the bottom.

> Use `ObjectSpec::card(p(1), "Card Alpha").in_zone(ZoneId::Library(p(1)))` and give each a
> `with_types(vec![CardType::Creature])` so `matches_filter` has something to chew on.
> **Gotcha**: `ObjectSpec::card()` creates naked objects — if any assertion depends on characteristics
> beyond name/type, call `enrich_spec_from_def()`.

### Tests

**`test_probe_draw_and_scry_agree_on_top`** — *CR 121.1, CR 701.22a*
1. Snapshot the library vector.
2. `draw_card(&mut state, p(1))`; record the drawn card's **name** → `drawn_name`.
3. On a **fresh clone** of the pre-draw state (`process_command` / draw take ownership — clone
   before each call, per the behavioral gotcha), execute `Effect::Scry { player: PlayerTarget::Controller, count: EffectAmount::Fixed(1) }`.
4. Determine which card Scry touched: after the scry, the looked-at card has been moved. Assert on
   **position**: the card Scry acted on must be the one named `drawn_name`.
   Concretely — assert that the card **still at index 0** of the library after `Scry 1` is
   `"Card Alpha"` (untouched bottom), i.e. Scry did **not** disturb the bottom.
5. `assert_eq!(drawn_name, "Card Gamma", "CR 121.1: draw takes the last element (Zone::top())");`

**`test_probe_reveal_and_route_sees_the_drawn_card`** — *CR 121.1*
Same 3-card library. Execute
`Effect::RevealAndRoute { count: EffectAmount::Fixed(1), matched_dest: Hand, unmatched_dest: Library{Bottom}, filter: <matches everything> }`
with a filter that matches all three cards. Assert **`"Card Gamma"` is the card now in hand** —
the same card `draw_card` yields. Cite CR 121.1.

**`test_probe_surveil_mills_the_drawn_card`** — *CR 701.25a, CR 121.1*
Same library. `Effect::Surveil { count: Fixed(1) }`. Assert the graveyard contains **`"Card Gamma"`**
and **not** `"Card Alpha"`.

### Pre-fix failure capture (MANDATORY EVIDENCE)

Before touching any non-test file, run:

```
~/.cargo/bin/cargo test -p mtg-engine --test mechanics_m_z library_ordering -- --nocapture
```

**All three probe tests MUST FAIL.** Expected pre-fix symptom: `RevealAndRoute` puts `"Card Alpha"`
in hand; `Surveil` mills `"Card Alpha"`; `Scry` disturbs index 0. **Paste the verbatim failure output
into the PB close-out and into `memory/primitives/pb-review-RS1.md` as the pre-fix evidence block.**
If any probe **passes** pre-fix, STOP — the defect model is wrong and the plan must be re-derived.

These tests are **kept permanently** as regressions. They are the canary the existing
`reveal_and_route.rs` suite failed to be.

---

## 4. Engine Change 1 — `Zone::top_n`

**File**: `crates/card-types/src/state/zone.rs`
**Action**: add **one** method immediately after `top()` (which ends at `:164`), before
`push_front` (`:165`).

```rust
/// Get the top `n` objects of an ordered zone, ordered from the top down.
///
/// Index 0 of the returned vector is the topmost card — the same card
/// `Zone::top()` returns and the same card a draw takes (CR 121.1).
/// Because ordered zones store the top at the LAST index, this walks the
/// backing vector in reverse.
///
/// Returns fewer than `n` entries if the zone is smaller (CR 401.7-adjacent:
/// callers must tolerate a short read). Returns empty for unordered zones,
/// matching `top()`.
pub fn top_n(&self, n: usize) -> Vec<ObjectId> {
    match self {
        Zone::Ordered(v) => v.iter().rev().take(n).copied().collect(),
        Zone::Unordered(_) => Vec::new(),
    }
}
```

**Contract (all three are load-bearing; assert them in unit tests)**
1. `top_n(1)` == `top().into_iter().collect()` — agreement with `top()` is the whole point.
2. Index 0 is topmost; the vector is ordered **top → down**.
3. `n > len` returns exactly `len` entries (no panic, no padding).
4. Unordered zones return empty (consistent with `top()` returning `None`).

**Unit tests** (co-located, `#[cfg(test)] mod tests` in `zone.rs` if one exists — otherwise add one):
- `test_top_n_agrees_with_top` — CR 121.1
- `test_top_n_orders_top_first`
- `test_top_n_over_length_saturates`
- `test_top_n_unordered_is_empty`

> **Why one helper and not four `.rev()` calls**: the defect *is* that this logic was open-coded four
> times and drifted from `top()` in every copy. Four fresh copies would reproduce the failure mode.
> Do not inline.

---

## 5. Engine Change 2 — rewire the four reads and the writes

### 5a. The four reads (`crates/engine/src/effects/mod.rs`)

Each of the four is the identical block:

```rust
let top_ids: Vec<ObjectId> = state
    .zones
    .get(&lib_zone)
    .map(|z| z.object_ids())
    .unwrap_or_default()
    .into_iter()
    .take(n)
    .collect();
```

Replace **each** with:

```rust
let top_ids: Vec<ObjectId> = state
    .zones
    .get(&lib_zone)
    .map(|z| z.top_n(n))
    .unwrap_or_default();
```

| # | Effect | Read site | Comment line to correct |
| --- | --- | --- | --- |
| 1 | `Effect::Scry` | `:3082-3090` | `:3082` — keep "ordered from top", it is now **true**. **Also fix `:3075`: `CR 701.18` → `CR 701.22`** (Finding A). |
| 2 | `Effect::Surveil` | `:3117-3125` | `:3117` — now true. `CR 701.25` at `:3105` is already correct; leave it. |
| 3 | `Effect::RevealAndRoute` | `:4985-4993` | `:4985` — now true. |
| 4 | `Effect::LookAtTopThenPlace` | `:5065-5073` | `:5065` says "same convention as RevealAndRoute" — **rewrite** to name `Zone::top_n` and cite CR 121.1. Also `:5045` says "same `object_ids().take(n)` top-N convention" — **stale, must be updated**. |

> **DO NOT DROP SURVEIL.** It is the byte-identical pattern. Fixing three of four would leave Surveil
> inverted *against the newly-corrected* `top_n` — strictly worse than today's uniform wrongness.

**Downstream ordering note**: all four arms subsequently `sort_by_key(|id| id.0)` for determinism
(`:3093`, `:3129`, `:5015-5016`, `:5165`). That sort is **unaffected** by this change and stays —
it governs *which* card wins ties, not *which end* was read. Do not remove it.

### 5b. The Scry bottom-write (the one direct site)

**File**: `crates/engine/src/effects/mod.rs:3094-3098`
**Current** (wrong — `push_back` = camp-A **top**):
```rust
for id in to_bottom {
    let _ = state.expect_move_object_to_zone(id, lib_zone);
}
```
**Replace with** `move_object_to_bottom_of_zone` (`state/mod.rs:1787`, `push_front`):
```rust
for id in to_bottom {
    // CR 701.22a: scried cards go on the BOTTOM. `expect_move_object_to_zone`
    // appends (= the top end under Zone::top()); the bottom requires push_front.
    let _ = state.move_object_to_bottom_of_zone(id, lib_zone);
}
```
Also correct the misleading comment at `:3095-3096` ("library zones are Ordered, so we use
move_to_zone back") — that comment is what rationalized the bug.

> **Check the exact name/signature/error-type of `move_object_to_bottom_of_zone` at
> `state/mod.rs:~1740-1802` before editing** and match the existing call sites in
> `rules/resolution.rs:5935` / `:6014` and `rules/copy.rs:484-495`. If it returns
> `Result<(ObjectId, Object), GameStateError>`, mirror how those cascade sites discard/handle it,
> and respect **SR-4** (pick a side: `expect_*` for engine-bug, `lki_*` for LKI-fizzle). Scry's
> existing `let _ =` is a silent discard — **prefer the `expect_*`-shaped variant** if one exists,
> since a failure here is an engine bug, not an LKI fizzle.

### 5c. ⚠️ RevealAndRoute / LookAtTopThenPlace — resolution of Finding B

**Surveil**: no bottom-write exists (writes to graveyard, CR 701.25a). **No action. Report this.**

The other two route through `resolve_zone_target`, which discards `LibraryPosition` at `:7830`. The
full fix is **explicitly out of scope**. Two options; **implement Option 1**:

**OPTION 1 (RECOMMENDED — narrow, does not widen the PB)**
In **each of those two arms only**, dispatch locally on the destination before moving. Do **not**
modify `resolve_zone_target`, and do **not** add a general `LibraryPosition` read path.

At `effects/mod.rs:5033-5039` (RevealAndRoute unmatched) and `:5166-5171` (LookAtTopThenPlace rest):

```rust
// CR 121.1 / 401.4: a Library{Bottom} destination means the FRONT of the
// ordered vector (Zone::push_front), not the append end. `resolve_zone_target`
// erases `position`, so this arm reads it directly. Narrow, local read —
// the general LibraryPosition capability gap is filed as a follow-up seed.
let to_bottom = matches!(
    unmatched_dest,
    ZoneTarget::Library { position: LibraryPosition::Bottom, .. }
);
```
then branch: `if to_bottom { state.move_object_to_bottom_of_zone(id, zone) } else { state.expect_move_object_to_zone(id, zone) }`.

Preserve the existing `new_id` handling and `zone_move_event(...)` emission on **both** branches —
both helpers return the same `(new_id, old_object)` shape.

**Justification**: this is squarely within "route the bottom-writes correctly," is ~6 lines in two
arms, adds no type/field, and leaves `resolve_zone_target`, `LibraryPosition`'s zero-read status,
and every other `ZoneTarget` consumer untouched. It closes the *behavioral* half of the defect
these two arms exhibit.

**OPTION 2 (FALLBACK, only if Option 1 proves entangled)**: read-side fix only for these two arms;
file the entire write side under the follow-up seed (§9). **Costs**: `LookAtTopThenPlace`'s
"Bottom the rest (CR 401)" comment at `:5158` stays a lie, and after the read fix the arm reads the
true top N and returns the rest to the true top — a near-no-op churn instead of a bottoming.
**Acceptable but strictly weaker.** If the runner takes Option 2, it **must** say so explicitly in
the close-out and update `:5158`'s comment to admit the gap.

**The reviewer arbitrates.** Either way, §0.3's finding (that the spec's "siblings share the shape"
premise is false) is a required close-out line item.

### 5d. Harness reconciliation

**File**: `crates/engine/src/testing/replay_harness.rs:207-212`

Today: comment says "top-to-bottom order"; the loop `builder.object(...)` **appends**, so the
**first-declared** card lands at index 0 = the **bottom**. The comment and the behavior contradict.

Under §2's decision the comment states the correct *intent* — script authors have been writing
libraries top-first (confirmed:
`test-data/generated-scripts/baseline/009_read_the_bones_scry_draw.json:73-76` lists Lightning Bolt
first, and `:167` narrates "p1 draws 2 cards (Lightning Bolt and Forest **from top of library**)").
**So the intent is right and the insert order is wrong.**

**Fix**: reverse the per-player insertion so the first-declared card ends up last in the vector:

```rust
// Add library cards. Scripts declare libraries TOP-TO-BOTTOM, but ordered
// zones store the top at the LAST index (Zone::top(), CR 121.1) — so insert
// in reverse to make the first-declared card the one draw_card yields.
for (owner_name, lib_cards) in sorted_zone_entries(&init.zones.library) {
    if let Some(&owner) = player_map.get(owner_name) {
        for card in lib_cards.iter().rev() {
            builder = builder.object(make_spec(owner, &card.card, ZoneId::Library(owner)));
        }
    }
}
```

> **SR-9b hazard**: `build_initial_state` must remain deterministic. `sorted_zone_entries` still
> governs cross-player ordering; only the within-player order changes. **`.rev()` on a slice is
> deterministic** — no ordering nondeterminism is introduced. But this **does** change ObjectId
> assignment order within a library, which will move per-step fingerprints. See §8.

---

## 6. Tests 2–5

All new tests live in **existing grouped targets** (SR-9a). Every test cites its CR rule (Invariant #8).

### Test 2 — de-vacuous `reveal_and_route.rs`

**File**: `crates/engine/tests/mechanics_m_z/reveal_and_route.rs`
**Sites**: `:83-104`, `:150-156`, `:223-237`, `:270`

These use **2-card libraries with `count: 4`** and assert membership/counts only — they pass today
while the bug is live. For each:
1. Grow the library to **≥5 cards** (strictly longer than `count: 4`) so the top-N read is a real
   *selection*, not a whole-library sweep.
2. Replace `hand_count == 2`-style membership assertions with **position** assertions: name the
   specific cards that must be routed, and assert the untouched remainder is still at the bottom
   indices in its original relative order.
3. Add to each test's doc comment: `/// CR 121.1: "top" is the end draw_card takes.`
4. Fix the stale `CR 701.16a` citation at `:108` if it does not resolve (verify via MCP; it is not
   Scry/Surveil/RevealAndRoute-related and looks like drift).

**Assertion shape** (use throughout):
```rust
let lib: Vec<String> = state.zones().get(&ZoneId::Library(p(1))).unwrap()
    .object_ids().iter()
    .map(|id| state.objects().get(id).unwrap().characteristics.name.clone())
    .collect();
// Vector is bottom→top; reverse for readability, then assert top-down.
assert_eq!(lib.last().unwrap(), "Expected Top Card", "CR 121.1");
```

### Test 3 — cascade round-trip

**File**: `crates/engine/tests/mechanics_m_z/library_ordering.rs`
**Test**: `test_cascade_bottomed_card_is_not_seen_by_next_scry` — *CR 702.85a + CR 701.22a + CR 121.1*

1. Library of ≥4 distinguishable cards.
2. Drive a cascade that bottoms a known card (`"Cascade Reject"`) — either via the real cascade path
   (`rules/resolution.rs:5935`) or, if wiring a full cascade is heavy, call
   `state.move_object_to_bottom_of_zone(...)` directly and **cite CR 702.85a** as the behavior being
   modeled. Prefer the real path; note the substitution if used.
3. Assert `"Cascade Reject"` is at library index 0.
4. Execute `Effect::Scry { count: Fixed(1) }`.
5. **Assert the Scry did NOT touch `"Cascade Reject"`** — it must still be at index 0.

This is the test that proves the two camps are reconciled. It is the single highest-value assertion
in the PB.

### Test 4 — scry-to-bottom ordering

**File**: same
**Test**: `test_scry_two_to_bottom_lands_below_everything` — *CR 701.22a + CR 121.1*

1. Library of 5: `[Bottom1, Bottom2, Bottom3, Top2, Top1]` (vector order; `Top1` is topmost).
2. `Effect::Scry { count: Fixed(2) }` — the deterministic fallback bottoms both.
3. Assert `Top1` and `Top2` are now at indices **0 and 1** (below every pre-existing card).
4. Assert `Bottom1/2/3` are now at indices 2/3/4 in their original relative order.
5. Assert `draw_card` now yields **`Bottom3`** — the new topmost.

> Step 5 is the cross-check that the write side and the read side agree. Without it, a
> compensating double-inversion would pass.

### Test 5 — golden-script reconciliation

**Survey method** (the runner must run all three, not just the first):
```
rg -l '"library"' /home/skydude/projects/scutemob/.worktrees/scutemob-143/test-data/generated-scripts
rg -n 'zones\.library' /home/skydude/projects/scutemob/.worktrees/scutemob-143/test-data/generated-scripts
rg -ni 'top of library|bottom of library|scry|surveil' /home/skydude/projects/scutemob/.worktrees/scutemob-143/test-data/generated-scripts
```

**Preliminary survey (mine, to be confirmed and extended by the runner):** most scripts assert
`zones.hand.p1.count` / `zones.library.p1.count` — **counts, not identities** — and are therefore
insensitive to the harness reversal. The at-risk set is scripts whose assertions or `note` fields
name **specific cards** drawn or scried. Confirmed candidates to inspect **first**:

| Script | Why |
| --- | --- |
| `baseline/009_read_the_bones_scry_draw.json` | Scry 2 + draw 2; `:167` names the drawn cards; `:26` carries the **wrong `CR 701.18`** citation → correct to **CR 701.22** |
| `stack/071_consider_surveil_then_draw.json` | Surveil then draw — both sides of the defect |
| `stack/005_brainstorm_draw_put_back.json` | Explicit put-back-on-top semantics |
| `stack/034_brainstorm_then_fetch.json` | Library manipulation + fetch |
| `baseline/113_mist_intruder_ingest_exile.json` | Exiles from the top |
| `stack/199_sakura_tribe_elder_search.json` | Library search + shuffle |
| `stack/204_cloak_cryptic_coat.json` | Top-of-library manipulation |

**Protocol per affected script**: update the assertion to the CR-correct behavior, add/correct
`cr_sections_tested` to include **`121.1`** and **`701.22`** (Scry) / **`701.25`** (Surveil), and
record the change in the close-out. **The reviewer must confirm no script silently encodes the old
inversion** — a script that "still passes" is not evidence of correctness here, exactly as
`reveal_and_route.rs` demonstrates.

> 🚫 **REPLAY-VIEWER OOM — HARD RULE.** Do **NOT** start the replay-viewer HTTP server to validate
> scripts. Agent-launched HTTP binaries get SIGKILL (137). Validate with:
> ```
> SCRIPT_FILTER=<script_name_without_ext> ~/.cargo/bin/cargo test --test scripts run_all_scripts -- --nocapture
> ```
> **`SCRIPT_FILTER=X matched 0 scripts` means a serde parse failure, not a filter miss** — the harness
> only runs `review_status: Approved` scripts, and a malformed `disputes` entry silently kills the
> parse. Set `"disputes": []` when editing a script.

---

## 7. Roster sweep — from `all_cards()`, NOT grep (SR-34/36)

**The count is a deliverable.** Grep baseline for calibration only: **47 distinct files**
(Scry 20, RevealAndRoute 18, Surveil 9, LookAtTopThenPlace 3, overlapping). Do **not** ship the
grep number as the answer.

`all_cards()` is build-generated (`crates/card-defs/build.rs:47-56`) and returns
`Vec<CardDefinition>` covering every registered def — this is why enumeration beats grep (grep
misses macro-generated and re-exported defs and over-counts comments).

**Method** — add a `#[test]` in an existing grouped target (suggest
`crates/engine/tests/core/`, alongside the other roster/inventory gates; **check for an existing
roster-sweep test there and follow its shape**):

1. `for def in mtg_card_defs::all_cards()`.
2. Walk **every** `Effect` tree reachable from the def: `abilities[..].effects`, triggered-ability
   effects, activated-ability effects, `modes`, and nested `Effect::ForEach` / `Effect::Conditional`
   / sequence bodies. **A shallow top-level scan will under-count** — nesting is where the
   grep/enumerate delta lives.
3. Match on `Effect::Scry { .. } | Effect::Surveil { .. } | Effect::RevealAndRoute { .. } | Effect::LookAtTopThenPlace { .. }`.
4. Collect `def.name` into a sorted, de-duplicated `Vec<String>`.
5. Emit the full list with `--nocapture` and assert a non-zero floor (e.g. `>= 40`) so the sweep
   cannot silently go vacuous.

**Deliverable**: the sorted list + per-effect counts + total, pasted into the close-out and
`pb-review-RS1.md`. Reconcile against the 47 grep baseline and **explain any delta** (expected:
enumeration ≥ grep).

Known members to sanity-check the sweep against: `goblin_ringleader`, `coiling_oracle`,
`sylvan_messenger`, `risen_reef`, `chaos_warp`, `satyr_wayfinder`, `birthing_ritual`,
`growing_rites_of_itlimoc`, `yuriko_the_tigers_shadow`, `six`.

### 🚫 NO CARD-DEF MARKER FLIPS IN THIS PB

This PB repairs **behavior**. Zero `completeness` changes, zero card-def edits. Several affected
defs are already `Complete` and were live-wrong; they become live-**right** without a marker change.
Flips land in follow-up authoring. **Any card-def diff in this PB is a scope violation.**

---

## 8. Wire-change verification — **NO BUMP** ✅

**Verified**: this PB adds and reshapes **nothing** on the wire.

| Closure | Change? | Evidence |
| --- | --- | --- |
| `Effect` variants/fields | **none** | Scry / Surveil / RevealAndRoute / LookAtTopThenPlace keep identical shapes; only the executor bodies change |
| `Command` | **none** | untouched |
| `GameEvent` | **none** | `Scried`, `Surveilled`, and `zone_move_event` outputs keep identical shapes |
| DSL / `card-types` public types | **`Zone::top_n` added** | a **method**, not a type/field/variant. Methods are outside the schema closure. `Zone`'s data shape is unchanged. |
| `LibraryPosition` / `ZoneTarget` | **none** | Option 1 (§5c) *reads* an existing field; adds nothing |
| Card defs | **none** | §7 |

> ⇒ **`PROTOCOL_VERSION` stays 26** (`crates/engine/src/rules/protocol.rs:248`).
> ⇒ **`HASH_SCHEMA_VERSION` stays 63** (`crates/engine/src/state/hash.rs:578`).

**What WILL move — and this is expected, not a contradiction**: scenario/state **hash values** and
per-step fingerprints (SR-9b), because ObjectId assignment order within libraries changes (§5d) and
cards now move to different indices. **The pinned artifacts are schema *fingerprints*, not state
*digests*.** Re-baseline any pinned per-scenario digest; do **not** bump the two schema constants.

> 🛑 **STOP CONDITION.** If the runner finds that `PROTOCOL_SCHEMA_FINGERPRINT` or the `HashInto`
> allowlist must be **re-pinned** (as opposed to scenario digests being re-baselined), that
> **contradicts this plan and the binding spec**. Do not plan around it: **halt, state it
> prominently in `pb-review-RS1.md`, and escalate for re-scoping.** A required schema re-pin means
> something in the closure moved that this analysis did not find.

---

## 9. Out of scope — file, do not fix

### Follow-up seed to FILE (do not implement): **OOS-RS1-1 — `LibraryPosition` is inert**

- `resolve_zone_target` discards `ZoneTarget::Library { position }` (`effects/mod.rs:7830`).
- `LibraryPosition` has **zero** engine read sites — only re-exports (`engine/src/cards/mod.rs:28`,
  `engine/src/lib.rs:12`) and a `HashInto` arm (`state/hash.rs:5474-5479`).
- Every `position: LibraryPosition::Bottom` in every card def is **inert decoration**.
- **Consequence**: Muxus's "rest on the bottom in a random order" is inexpressible; this is why
  **OOS-OS8-2 (muxus authoring) is gated behind PB-RS1 and remains gated after it.**
- §5c Option 1 gives two arms a *local* bottom-write; it does **not** close this seed.

**File it** into `memory/primitives/rider-seed-triage-2026-07-19.md` §1c (new-seed table) with class
**capability**, and cross-reference from the PB-RS1 close-out.

### Also out of scope
- **Any card-def marker flip** (§7).
- **Authoring muxus** — gated on this PB, not part of it.
- **Changing Scry's all-N-to-bottom fallback** (§0.4) — legal under CR 701.22a's "any number."

---

## 10. Exhaustive-match / build gotchas

**Expected to be a non-issue** — this PB adds no enum variant, so the usual exhaustive-match tax
should not apply. **Verify anyway**, because runners miss these ~50% of the time:

| File | Match on | Expected action |
| --- | --- | --- |
| `tools/tui/src/play/panels/stack_view.rs` | `StackObjectKind` (exhaustive) | **none expected** — no new variant |
| `tools/replay-viewer/src/view_model.rs` | `StackObjectKind` **and** `KeywordAbility` (exhaustive) | **none expected** |
| `crates/engine/src/state/hash.rs` | `HashInto` allowlist | **none expected** — no new field |

> ✅ **`~/.cargo/bin/cargo build --workspace` after the implement phase is MANDATORY.** The TUI and
> replay-viewer are **not** covered by `cargo check -p mtg-engine`. This is the single most-missed
> step in this pipeline.

**Other gotchas in play**
- **SR-6**: `mtg-card-defs` depends on `card-types` only. `Zone::top_n` lands in **`card-types`**, so
  the 1,798 defs **will** rebuild. That is expected and correct — not a layering violation.
- **SR-35**: `cargo fmt --check` checks **zero** of the card defs. Run **`tools/check-defs-fmt.sh`**
  too (or rely on `cargo test --all`, which runs it via `core card_defs_fmt`).
- **SR-4**: new failure paths in `effects/mod.rs` must pick a side — `expect_*` (engine bug) vs
  `lki_*` (LKI fizzle). Scry's bottom-write failure is an **engine bug** → `expect_*`.
- `process_command()` / draw take ownership of `GameState` — **`.clone()` before each call** in the
  probe's comparative tests.
- `ObjectSpec::card()` creates naked objects — call `enrich_spec_from_def()` if characteristics matter.
- **DrawCards on an empty library is silently a no-op** — every library in every new test needs
  enough cards for the assertions.

---

## 11. Execution order (the runner follows this literally)

1. **Probe first.** Write `tests/mechanics_m_z/library_ordering.rs` + the `mod` line (§3).
   Run it. **Capture the verbatim failure output.** Do not edit non-test files before this.
2. Add `Zone::top_n` + its 4 unit tests (§4). Probe still fails (nothing is wired yet).
3. Rewire the four reads + correct the four comments + **CR 701.18 → 701.22** (§5a).
4. Fix Scry's bottom-write (§5b).
5. Apply §5c **Option 1** to RevealAndRoute + LookAtTopThenPlace. Record Finding B in the close-out.
6. Reconcile the harness (§5d).
7. Re-run the probe. **All three probe tests must now PASS.** Capture the post-fix output.
8. Tests 3 and 4 (§6).
9. De-vacuous `reveal_and_route.rs` (Test 2, §6).
10. Golden-script survey + reconciliation (Test 5, §6). **Never start the HTTP server.**
11. Roster sweep test + emit the list (§7).
12. File seed **OOS-RS1-1** (§9). **Fix nothing.**
13. Gates (§12).
14. Close-out (§13).

---

## 12. Verification Checklist

- [ ] Probe written FIRST; **pre-fix failure output captured verbatim** and pasted into the close-out
- [ ] `Zone::top_n` added in `card-types/src/state/zone.rs` next to `top()`, with 4 contract unit tests
- [ ] All **FOUR** reads rewired (Scry, **Surveil**, RevealAndRoute, LookAtTopThenPlace) — none dropped
- [ ] All four stale "ordered from top" comments corrected; `:5045` and `:5065` de-staled
- [ ] **`CR 701.18` → `CR 701.22`** corrected at `effects/mod.rs:3075` and in affected scripts
- [ ] Scry bottom-write routed through `move_object_to_bottom_of_zone`
- [ ] Finding B (Surveil has no bottom-write; two arms gated on `LibraryPosition`) **reported** in close-out
- [ ] §5c Option 1 applied (or Option 2 taken **and explicitly declared**)
- [ ] Harness `:207-212` comment + insert loop reconciled
- [ ] Probe now PASSES; post-fix output captured
- [ ] Tests 2–5 complete; every test cites its CR rule (Invariant #8)
- [ ] New test file has its `mod` line in `mechanics_m_z/main.rs` (SR-9a); **no** top-level `tests/*.rs`
- [ ] Roster sweep run from **`all_cards()`**; full sorted list + counts in close-out; delta vs 47 explained
- [ ] **Zero card-def diffs** (`git diff --stat crates/card-defs/` is empty)
- [ ] `PROTOCOL_VERSION` still **26**, `HASH_SCHEMA_VERSION` still **63**
- [ ] Seed **OOS-RS1-1** filed, not fixed
- [ ] `~/.cargo/bin/cargo test --all` green
- [ ] `~/.cargo/bin/cargo clippy --all-targets -- -D warnings` clean
- [ ] `~/.cargo/bin/cargo fmt --check` **and** `tools/check-defs-fmt.sh` clean (SR-35)
- [ ] **`~/.cargo/bin/cargo build --workspace`** green (TUI + replay-viewer)

---

## 13. Close-out deliverables

1. Pre-fix and post-fix probe output (verbatim).
2. **The roster count** — full sorted list from `all_cards()`, per-effect counts, total, delta vs 47.
3. **Finding A** — the CR 701.19 / 701.18 → **701.22** citation correction (spec + source + script).
4. **Finding B** — "the siblings share the shape" is false; the 1/1/2 breakdown; which option taken.
5. The golden-script reconciliation list.
6. Seed **OOS-RS1-1** filed.
7. Confirmation that PROTOCOL/HASH did **not** move, and which scenario digests were re-baselined.

---

## 14. Risks & Edge Cases

| Risk | Mitigation |
| --- | --- |
| **Harness reversal breaks scripts that assert card identity.** Changing library insert order changes which cards are drawn in ~210 approved scripts. | Most assert **counts**, not identities — low blast radius, but **not zero**. Run the full script suite early (step 6→7 boundary), not at the end. Any script that flips is evidence the old inversion was baked in — fix the script with a CR citation, do not revert the harness. |
| **A compensating double-inversion passes tests while staying wrong.** Fixing reads and writes in the same commit can mask an error in one. | Test 4 step 5 (`draw_card` yields the new top after a scry-to-bottom) is the designed cross-check. Test 3 (cascade↔scry) crosses subsystems. |
| **Finding B pulls the PB toward the out-of-scope `LibraryPosition` fix.** | §5c Option 1 is deliberately local and adds no type. If the runner finds itself editing `resolve_zone_target`, **stop** — that is the out-of-scope seed. |
| **`sort_by_key(|id| id.0)` interacts with the new read order.** | The sort is orthogonal (tie-break within the selected set, not selection). Leave it. Tests 2/4 assert positions, which would catch it if wrong. |
| **`.rev()` and SR-9b determinism.** | `.rev()` on an ordered slice is fully deterministic. Cross-regime fingerprints (SR-9b) will move but stay *equal across regimes* — verify both regimes move together; if they diverge, that is a real bug. |
| **Scry with `n > library.len()`.** | `top_n` saturates (contract 3). Add a short-library case to Test 4. Interacts with CR 401.7. |
| **Surveil 0 / Scry 0.** | CR 701.22b and 701.25c: no event. Surveil already guards at `:3112`; `top_n(0)` returns empty. Verify Scry 0 still emits/suppresses as before — **this PB must not change that behavior.** |
| **Unordered zones.** | `top_n` returns empty, matching `top()` → `None`. All four call sites use `ZoneId::Library`, always `Ordered`. |
| **`card-types` change rebuilds all 1,798 defs.** | Expected under SR-6, not a violation. Budget build time; do not "optimize" by moving `top_n` into the engine — it must sit next to `top()`. |

---

## 16. Pre-fix probe evidence (captured 2026-07-19, before any production edit)

Command: `~/.cargo/bin/cargo test -p mtg-engine --test mechanics_m_z library_ordering -- --nocapture`

All 5 tests that depend on the fix FAILED, as required by acceptance criterion 5110
(the step-0 probe tests plus Tests 3 and 4, written together before any production edit):

```
thread 'library_ordering::test_probe_surveil_mills_the_drawn_card' panicked at crates/engine/tests/mechanics_m_z/library_ordering.rs:190:5:
CR 701.25a: Surveil 1 should mill the top card (Card Gamma), got ["Card Alpha"]

thread 'library_ordering::test_scry_two_to_bottom_lands_below_everything' panicked at crates/engine/tests/mechanics_m_z/library_ordering.rs:538:5:
assertion `left == right` failed: CR 701.22a: both scried cards must land below every pre-existing card, got ["Bottom3", "Top2", "Top1", "Bottom1", "Bottom2"]
  left: ["Bottom3", "Top2"]
 right: ["Top1", "Top2"]

thread 'library_ordering::test_probe_reveal_and_route_sees_the_drawn_card' panicked at crates/engine/tests/mechanics_m_z/library_ordering.rs:161:5:
assertion `left == right` failed: CR 121.1: RevealAndRoute count:1 must see the same card draw_card yields
  left: "Card Alpha"
 right: "Card Gamma"

thread 'library_ordering::test_probe_draw_and_scry_agree_on_top' panicked at crates/engine/tests/mechanics_m_z/library_ordering.rs:125:5:
assertion `left == right` failed: CR 701.22a: Scry 1 must look at (and re-bottom) the same card draw_card takes
  left: "Card Beta"
 right: "Card Gamma"

thread 'library_ordering::test_cascade_bottomed_card_is_not_seen_by_next_scry' panicked at crates/engine/tests/mechanics_m_z/library_ordering.rs:468:5:
CR 701.22a: Scry 1 must not read or move the card cascade just bottomed (object identity should survive -- CR 400.7 says a NEW id means it was moved)

test result: FAILED. 0 passed; 5 failed; 0 ignored; 0 measured; 633 filtered out; finished in 0.00s
```

Note on the step-0 spec's illustrative assertion: §3 step 4 suggested asserting "the card still
at index 0 after Scry 1 is 'Card Alpha' (untouched bottom)." That specific wording cannot hold
under §0.4 (Scry's deterministic fallback always bottoms every card it looks at, even for
count 1) — bottoming via `push_front` always displaces whatever was previously at index 0 up to
index 1, so index 0 after ANY Scry-with-bottom-writes is the just-scried card, never the
previously-untouched bottom card, in both the pre-fix and post-fix engine. The implemented
assertion instead checks that index 0 equals `drawn_name` (the card `draw_card` would take) —
which satisfies the plan's primary instruction ("the card Scry acted on must be the one named
drawn_name") and correctly discriminates pre/post-fix, as shown above (pre-fix index 0 is
"Card Beta", post-fix it is "Card Gamma"). Test 3 similarly uses an ObjectId-identity check
(CR 400.7: a new id means the object was moved) rather than a position check, since
`move_object_to_bottom_of_zone`/`move_object_to_zone` are `pub(crate)` and unavailable to the
integration-test crate — the plan's fallback wording ("call state.move_object_to_bottom_of_zone
directly") isn't actually reachable from black-box tests, so the real cascade path (already used
by `mechanics_a_d/cascade.rs`) was used instead, exactly as the plan's own preference ranks it.

## 15. Why this is the right first dispatch

Highest correctness leverage in the R1–R11 queue: live-wrong on already-`Complete` cards
(Invariant #9); the **only** queue item whose defect silently corrupts *other* subsystems' results
(cascade actively feeds the wrong end to every scry/reveal card); engine-internal with **no wire
bump**; and it de-vacuouses a suite that passes today while the bug is live. Direct analogue of
PB-OS1's "de-vacuous the canary that lies." It is also a hard gate on OOS-OS8-2 — though note §9:
muxus stays gated even after this PB, on `LibraryPosition`.
