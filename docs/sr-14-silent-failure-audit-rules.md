# SR-14 — Silent-Failure Audit: the rest of `rules/`

<!-- last_updated: 2026-07-16 -->

> **SR-25 (`scutemob-80`, 2026-07-16) extends this audit** to the last unswept files —
> `rules/layers.rs`, `rules/commander.rs`, `rules/miracle.rs`, the four small
> foretell/plot/priority/suspend/turn_structure sites, and the non-primitive swallow-sites in
> `state/mod.rs` — and adds the **anti-regression ratchet** the discipline never had. See the
> [SR-25 section](#sr-25--the-last-unswept-files--the-anti-regression-ratchet) at the bottom.
> Note: SR-14's table below still spells the fizzle helper `lki_object[_mut]`; **SR-23 renamed
> that family to `fizzle_object[_mut]`** (the `lki_` prefix now means only the
> `lki_object_snapshot` store). SR-25 used the current names.

> Task: `scutemob-66`. Follow-up to `scutemob-56` (SR-4), which swept `effects/mod.rs`
> and `rules/resolution.rs`. This task applies the **same method and the same
> `state::diagnostics` vocabulary** to the ten remaining `rules/` files the SR-4 audit
> enumerated but left unswept. Read `docs/sr-4-silent-failure-audit.md` first — it is the
> method; this doc records only what SR-14 added and the two live surprises it turned up.

## Scope

Per acceptance criterion, the files swept are:

```
rules/{abilities, casting, combat, sba, replacement, turn_actions, mana, copy, engine, lands}.rs
```

`rules/layers.rs` and `rules/continuous_effect.rs` (named in the SR-4 audit's out-of-scope
list) are **not** in this task's criteria and were left alone; `layers.rs` is where
`calculate_characteristics` / `expect_characteristics` *live*, so its own internal calls are
a different concern.

## Method (unchanged from SR-4)

The classification is decidable because of three engine facts, re-stated here so this doc
stands alone:

1. **`GameState::players` never loses entries** — every missing `PlayerId` is a bug, so a
   player lookup is *always* `expect_player[_mut]`. Verified again for SR-14: `grep`
   across the whole engine finds no `players.remove`; the only hit is the explanatory
   comment in `state/diagnostics.rs`. This let SR-14 **retire a stale `MR-M4-01` comment**
   in `sba.rs::check_player_sbas` ("player may have been removed mid-pass") that
   contradicted the ground truth.
2. **`GameState::zones` never shrinks** — every missing `ZoneId` is a bug
   (`expect_zone[_mut]`). A `zones.get(..).and_then(|z| z.top())` returning `None` on an
   empty library is a *legitimate* empty-check, not a swallow, and was left alone.
3. **`calculate_characteristics(state, id)` returns `None` iff `id` is absent from
   `state.objects`** — so `.unwrap_or_else(|| obj.characteristics.clone())` (which
   substitutes *pre-layer* printed characteristics) is dead code where the id is live and
   the fizzle branch where it is not. Impossible sites became
   `layers::expect_characteristics`; fizzle sites keep `calculate_characteristics` and a
   `.map(..).unwrap_or(..)` that names the fizzle, with a CR citation.

Object lookups are classified by **ObjectId provenance**: read straight out of
`state.objects`/a zone with no intervening zone change, just returned by a move/add, or a
loop variable over live `state.objects` ⇒ IMPOSSIBLE (`expect_object[_mut]`); captured from
a resolved target, an ability's `ctx.source`, a stack payload, a trigger source, or a `Vec`
collected before a mutating/removing loop ⇒ FIZZLE (`lki_object[_mut]`, CR-cited).

The work was fanned out to nine parallel read-only-then-edit classifiers (one per file,
smallest grouped), each required to trace provenance per site and biased toward FIZZLE under
uncertainty — a wrong IMPOSSIBLE panics the suite loudly; a wrong FIZZLE merely fails to
place a tripwire. Two of those uncertain calls were adjudicated by the suite (below).

## Results

~360 sites converted across the ten files (lands.rs handled first, by hand, as the template).
Approximate vocabulary usage per file after conversion:

| File | `expect_object[_mut]` | `expect_characteristics` | `expect_player[_mut]` | `expect_zone[_mut]` | `expect_move_*` | `lki_object[_mut]` |
|---|---:|---:|---:|---:|---:|---:|
| abilities.rs | 19 | 29 | 8 | – | – | 37 |
| casting.rs | 5 | 25 | 2 | – | – | – |
| combat.rs | 11 | 4 | 16 | – | – | 1 |
| sba.rs | 8 | 6 | 14 | 1 | 7 | 8 |
| replacement.rs | 16 | 7 | 4 | 4 | 3 | 3 |
| turn_actions.rs | 22 | 6 | 11 | 2 | 9 | 4 |
| mana.rs | – | 4 | – | – | – | – |
| copy.rs | – | 4 | 3 | – | 5 | – |
| engine.rs | 9 | 6 | 11 | 1 | – | 4 |
| lands.rs | 12 | 2 | – | – | – | – |

(Counts are line-based approximations; `expect_object` includes `_mut`.)

The `lki_*` fizzle branches carry CR citations, checked mechanically: every `lki_object` /
`lki_move_object_to_zone` site in all ten files has a `CR NNN` comment within the preceding
few lines. The dominant citations are CR 400.7 (a zone change makes a new object, the old id
is dead), CR 608.2b (an effect can't find info about an illegal target), CR 113.7a (an
ability outlives its source ⇒ LKI), and CR 603.10a (dies/leaves-battlefield "look-back"
reads of the object in its new zone).

## Two IMPOSSIBLE verdicts the debug-assert suite demoted to FIZZLE

Both were flagged as *uncertain* by their classifier before the suite ran, exactly as the
SR-4 method predicts ("bias toward FIZZLE and let `cargo test --all` adjudicate"). Both
would have panicked a real game.

1. **`replacement.rs::queue_carddef_etb_triggers`** — `new_id` is a *caller-supplied*
   parameter, and the function is written throughout to tolerate its absence (three
   `.unwrap_or(ZoneId::Exile)` / `.unwrap_or_default()` / raw `.get(..).unwrap_or(false)`
   fallbacks). `test_fabricate_token_fallback` calls it with an id that has no live object,
   so the three `expect_object(new_id)` reads (face-down check + IG-1 suppressor scan) fired.
   Demoted to `lki_object` (CR 400.7): a permanent removed by an earlier same-batch ETB
   replacement simply has no ETB triggers to suppress.
2. **`casting.rs::has_split_second_on_stack`** — a `StackObjectKind::Spell`'s
   `source_object` was assumed to live in `state.objects` for as long as the stack entry
   does. `test_plot_free_cast_requires_empty_stack` proves a free/plotted cast can leave a
   `Spell` stack entry whose source object is gone. Demoted to
   `calculate_characteristics(..).map(has_split_second).unwrap_or(false)` (CR 400.7 /
   113.7a): a missing source cannot have Split Second.

These are the SR-14 analogue of SR-4's "believe the code over the classifier" rule — here
the *test* is the code that was believed.

## Disjoint-borrow sites

Three sites keep raw `state.objects.get_mut(&id)` guarded by `debug_assert_object_live!`
because the block also reads another `state` field while the object is borrowed
(`state.expect_object_mut(id)` would borrow all of `GameState`):

- `engine.rs` transform loop (reads `state.timestamp_counter`) — found by the engine classifier.
- `engine.rs::activate_loyalty_ability` loyalty payment (the `def`/`effect` locals hold a
  `state.card_registry` borrow) — this one the classifier *missed*; it surfaced as an E0502
  on the first workspace compile and was fixed to the `debug_assert_object_live!` form.
- (`turn_actions.rs` daybound/nightbound transform loop reads `state.timestamp_counter`.)

## Deliberately left alone (NONSWALLOW)

Per SR-4's taxonomy: `?`-propagation, handled `Err`/`None` match arms doing real work,
predicates `.map(..).unwrap_or(false)` over a possibly-absent object (where `false` is the
right answer when it is gone), scalar characteristic defaults (`power.unwrap_or(0)`, CR
208.2), `.unwrap()` guarded by a present-check, `let _ = x;`, and `.or_else`-chained
`Option<Characteristics>` that thread the `None` into an event payload rather than unwrapping
it to printed characteristics.

One judgment worth recording: in `combat.rs` the classifier converted **player**
predicates (`players.get(&pid).map(..).unwrap_or(false)`) to `expect_player` — ground truth
1 says a missing `PlayerId` is always a bug — but left **object** predicates of the same
shape as NONSWALLOW, because a departed object legitimately answers the predicate `false`.
This is the one place the "leave predicates alone" rule and ground-truth-1 pull opposite
ways; the split follows SR-4's precedent (its `PLAYERS_GET` table has zero NONSWALLOW rows).

## Out of scope, noted for follow-up

The `mana.rs` classifier flagged a latent SR-13-style LKI *semantics* gap (not a swallow):
`mana_source_matches` reads a tapped source's characteristics via
`calculate_characteristics(source).map(..).unwrap_or(false)`, so a card that both **taps and
sacrifices itself** for mana would fail to fire a subtype-filtered "whenever tapped for mana"
trigger (the source is gone by the time the trigger fires; CR 113.7a wants LKI). SR-14
preserved behavior and did **not** widen into it, exactly as SR-4 deferred wither/infect to
SR-13. File a new SR task if this is ever wanted.

## Verification

- `cargo test --all` (debug assertions ON): **3201 passed / 0 failed**, 29 suites.
- `cargo clippy --all-targets -- -D warnings`: clean.
- `cargo fmt --all -- --check`: clean.
- `cargo build --workspace`: clean (the invariant-#3 seal gate).

No assertion fires under the existing suite once the two demotions above are applied — the
~250 new `expect_*` asserts are tripwires for the next regression, exercised by whatever
coverage the suite already has.

---

# SR-25 — the last unswept files + the anti-regression ratchet

> Task: `scutemob-80`. Applies the SR-4 method (`docs/sr-4-silent-failure-audit.md`) and the
> **post-SR-23 `state::diagnostics` vocabulary** (`fizzle_object[_mut]`, not `lki_object[_mut]`)
> to the files SR-4/SR-14 enumerated but never swept, and closes the gap the re-audit flagged:
> the "new code must pick a side" discipline had **no machine check**, so the ~760 sites SR-4
> and SR-14 classified could regress invisibly.

## Two halves

1. **Sweep** the never-swept files.
2. **Ratchet** — a source-scan test pinning a per-file ceiling on bare
   `.objects/.players/.zones.get[_mut](` lookups across *all* swept files. A count may only
   decrease; a new bare lookup fails the suite with a message pointing at the vocabulary.

## Scope swept

`rules/layers.rs` (the characteristics hot path — `calculate_characteristics` lives here, which
is why SR-14 left it alone), `rules/commander.rs`, `rules/miracle.rs`,
`rules/{foretell,plot,priority,suspend,turn_structure}.rs` (one site each), and the
non-primitive sites in `state/mod.rs`.

## Conversions (28 total)

| File | → `expect_*` (IMPOSSIBLE) | → `fizzle_*` (FIZZLE) | Left NONSWALLOW / primitive |
|---|---|---|---|
| `layers.rs` | 9 (5 `expect_object`, 2 `expect_object_mut`, 1 `expect_player_mut`, 1 `expect_player`) | – | 27 (see below) |
| `commander.rs` | 5 (2 `expect_zone`, 2 `expect_object`, 1 `expect_player_mut`) | – | 1 (empty-library `.and_then(z.top())`) |
| `miracle.rs` | 3 (1 `expect_player`, 2 `expect_object`) | – | – |
| `foretell.rs` / `plot.rs` / `suspend.rs` | 1 each `expect_object_mut` | – | – |
| `priority.rs` / `turn_structure.rs` | 1 each `expect_player` | – | – |
| `state/mod.rs` | 5 (1 `expect_object_mut`, 1 `expect_player_mut`, 2 `expect_zone_mut`) | 2 `fizzle_object_mut` | primitives (accessors, `zone()`, `objects_in_zone`, `face_down_reveal_for`) |

### `layers.rs` — the load-bearing judgment

The 45 single-line bare lookups split cleanly: **9 are IMPOSSIBLE re-reads** and **the rest are
NONSWALLOW predicates left alone**, exactly as SR-14's "leave object predicates alone" precedent
(a departed object legitimately answers a `.map(|o| ..).unwrap_or(false)` filter predicate
`false`).

- The five re-reads of `object_id` inside `calculate_characteristics` (lines 279/303/331/375/412)
  became `expect_object`. The function takes `&GameState` and holds the line-39 `obj` borrow
  live across its whole body, so the entry cannot vanish mid-function — a `None` is an engine
  bug. This **corrected a stale `MR-M5-01` comment** ("if-let instead of expect — object may
  have been removed by an effect") that contradicted the immutable-borrow ground truth.
- The untap-step reset loops (1637/1656) read ids collected from live `state.objects` one line
  above and mutate a single field → `expect_object_mut`; the active-player protection clear
  (1621) and the CDA poison-counter fold (1785) are player lookups → `expect_player_mut` /
  `expect_player` (ground truth 1).
- The ~35 `source_controller` / `obj_controller` dependency-filter reads (690–1012) and the six
  multi-line `ControlledBy` / `AttachedCreature` predicates stay bare: they are
  `.map(..).unwrap_or(false)` questions whose `false`-on-absent is the correct answer.
- `calculate_characteristics`'s own line-39 `state.objects.get(&object_id)?` stays bare **by
  construction** — it is the primitive whose "`None` iff absent" contract every `fizzle_*` /
  `expect_characteristics` caller depends on (ground truth 3). Asserting there would make every
  legitimate fizzle caller panic.

Perf: `expect_*` is `debug_assert!`-only, so the release characteristics hot path (the 23µs
`priority_cycle_4p` bench) is byte-for-byte a `self.objects.get`. No disjoint-borrow site arose
in this file (all re-reads are shared borrows; the two `_mut` loops touch only the looked-up
object).

### `state/mod.rs` — primitives vs. genuine swallow-sites

Most of this file's lookups are the **primitive accessors the whole vocabulary is built on**
(`object`, `player`, `zone`, `add_object`, `move_object_to_zone`, `objects_in_zone`) — leaving
them bare is correct; wrapping them in `expect_*` would be circular. The genuine swallow-sites:

- `add_object` set of `created_token_this_turn` on `object.controller` → `expect_player_mut`.
- `retimestamp_attached_source` (SR-30's new helper) re-timestamps `source_id`; all three
  callers pass a resolved-present equipment → `expect_object_mut`.
- The two meld-split zone inserts into the already-validated destination `to` → `expect_zone_mut`.
- The **two `paired_with` clears (CR 702.95e)** are genuine **FIZZLE**s → `fizzle_object_mut`:
  a Soulbond partner can leave the battlefield in the same SBA batch (CR 400.7 retires its id),
  so a missing partner is a legal do-nothing, not corrupted state. This is the file's one
  bias-to-fizzle call.
- `face_down_reveal_for` (`self.objects.get(&object_id)?`) is left NONSWALLOW: it is a public
  query whose `None` ("no object → nothing to reveal") is a legitimate answer, of a piece with
  the zone/face-down guards that follow it.

## The ratchet — `crates/engine/tests/core/bare_lookup_ratchet.rs`

A source-scan test (in the `core` group, like `lki_diagnostics_scan`) that pins a **per-file
ceiling** for all 21 swept files (SR-4's 2 + SR-14's 10 + SR-25's 9). Design choices, each
closing a specific evasion:

- **Comment-stripped** (`//`-to-EOL removed) so a doc comment quoting `.objects.get(` cannot
  inflate a ceiling (it did: `casting.rs` 39→34, `engine.rs` 26→24 once comments were stripped).
- **Whitespace-insensitive** (all whitespace removed before matching) so the count is
  rustfmt-stable *and* un-evadable by line-splitting: a multi-line
  `state\n .objects\n .get(&id)` chain counts identically to the inline form. (This is why the
  ceilings — e.g. `effects/mod.rs` 100, `resolution.rs` 102, `layers.rs` 51 — are far above the
  single-line grep counts: they include the many multi-line NONSWALLOW predicate reads.)
- **Exact-match, not just a cap.** Over the ceiling → regression (fails, points at the
  diagnostics vocabulary). *Under* the ceiling → also fails, asking you to lower the pin, so no
  slack accumulates for a future regression to hide beneath.
- **Denominator guards**: a floor on the roster size (`MIN_FILES = 21`, so the list can't be
  gutted to a few green files), a floor on the aggregate count (`MIN_TOTAL = 400`, live total
  **477** — a counter broken to return 0 would collapse below it), and a per-file `len() > 200`
  read-proof so a mis-pathed 0 can't pass a 0-ceiling vacuously.
- **Counter self-tests** (`counter_is_non_vacuous`): proves the counter sees inline lookups,
  distinguishes `get` from `get_mut`, ignores comments, is blind to line-splitting, and does not
  false-match `stack_objects` / `lki_objects` (leading `_`, not `.`).
- **Known limitation** (shared with the SR-5/SR-8 scans): only `//` line comments are stripped,
  not block comments, so a contrived `state.objects/**/.get(&id)` would evade the needle. Not a
  realistic regression path (clippy/review would reject it); documented, not defended against.
- **Vocabulary-exists guard** (`diagnostics_vocabulary_still_exists`): the failure message names
  `expect_object` / `fizzle_object` / …; this asserts those `fn`s still exist so the message
  can't rot into pointing at a renamed getter (the SR-23 hazard, one level up).

### Adversarial demonstration

Injecting `let _sr25_demo = state.objects.get(&new_exile_id);` into `foretell.rs` (ceiling 0)
— edit confirmed present in the file — reddened the ratchet with
`src/rules/foretell.rs now has 1 bare … lookups, up from the pinned 0`, then reverting restored
green. So the gate is non-vacuous: it catches a new bare lookup in a fully-swept file.

## Verification

- Full sweep + ratchet: `cargo test --all`, `cargo clippy --all-targets -- -D warnings`,
  `cargo fmt --all -- --check`, `cargo build --workspace` (the invariant-#3 seal gate) — all
  clean (see the task's completion record).
- No new `expect_*` assertion fires under the existing suite (debug assertions ON) — the ~28 new
  asserts are tripwires for the next regression, exercised by whatever coverage the suite has,
  exactly as in SR-4/SR-14.
