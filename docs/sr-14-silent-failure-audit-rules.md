# SR-14 — Silent-Failure Audit: the rest of `rules/`

<!-- last_updated: 2026-07-10 -->

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
