# SR-4 — Silent-Failure Audit: effects/mod.rs and rules/resolution.rs

<!-- last_updated: 2026-07-10 -->

> Task: `scutemob-56`. Companion to `docs/sr-remediation-plan.md`.
> Every line number in the per-site table below refers to the **pre-change tree at
> `ef0d9579`** (the branch point), so the classification can be re-derived against a
> stable snapshot. The code they describe now lives behind the vocabulary in
> `crates/engine/src/state/diagnostics.rs`.

## The problem

`if let Some(obj) = state.objects.get_mut(&id) { obj.tapped = true; }`

Read on its own, that line cannot tell you which of two very different things is
happening when the lookup misses.

**Expected fizzle.** `id` is *last known information* — a target that has since
changed zones, a sacrificed source, a creature that died to a state-based action
mid-resolution. CR 400.7 says an object that changes zones "becomes a new object with
no memory of, or relation to, its previous existence," so the old `ObjectId` names
nothing. CR 608.2b says an effect that needs information about such an object "fails
to determine any such information. Any part of the effect that requires that
information won't happen." **Doing nothing is correct.**

**Impossible absence.** `id` came out of `state.objects` three lines earlier, or it is
a `PlayerId`. Absence means an engine invariant is broken. Doing nothing silently
corrupts the game state and — worse — corrupts the state history that architecture
invariant #2 (rewind/replay) depends on. **Doing nothing is a bug that will surface
many turns later as an unexplainable replay divergence.**

Both compile to the same silent, `else`-less `if let`. The bug hides behind the
fizzle. SR-4 makes the distinction a property of the code rather than of a comment
that may or may not exist.

## Three engine facts that make the classification decidable

This is the part worth remembering. Without these, "is this absence possible?" is a
judgment call at 400 sites. With them, it is mostly mechanical.

1. **`GameState::players` never loses entries.** `rg 'players.remove'` finds nothing.
   A player who loses (CR 104.2/104.3) is flagged `PlayerState::has_lost`; under
   CR 800.4a their *objects* leave the game, but the `PlayerState` stays addressable
   so turn order, APNAP ordering and replay keep working. **Every missing `PlayerId`
   is a bug — there is no fizzle case.** That decided 32 sites outright.

2. **`GameState::zones` is built before turn 1 and never shrinks.** Every missing
   `ZoneId` is a fabricated id. That decided 8 more.

3. **`layers::calculate_characteristics` returns `None` if and only if the `ObjectId`
   is absent from `state.objects`.** Every other step in it is total. So its
   `unwrap_or_*` fallbacks are never "a partial computation we're rounding off" —
   they are either dead code (the id is live) or the fizzle branch (it isn't). There
   is no third possibility, which is what makes `expect_characteristics` safe.

Only `state.objects` genuinely loses entries — `move_object_to_zone` removes the old
id and inserts a new one (CR 400.7), and `sba.rs` removes dead objects. So an object
lookup is classified by tracing the **provenance of the `ObjectId`**: read straight
out of `state.objects` with no intervening zone change ⇒ impossible; captured from a
target, a stack payload, a trigger's source, or a `Vec` collected before a mutating
loop ⇒ fizzle.

## The vocabulary

Added in `crates/engine/src/state/diagnostics.rs` and `rules/layers.rs`. All the
`expect_*` forms carry `#[track_caller]`, so a violated assertion points at the call
site rather than at the helper.

| Absence means | Call | Behavior |
|---|---|---|
| engine bug | `expect_player`, `expect_player_mut` | `debug_assert!`; `None` in release |
| engine bug | `expect_object`, `expect_object_mut` | `debug_assert!`; `None` in release |
| engine bug | `expect_zone`, `expect_zone_mut` | `debug_assert!`; `None` in release |
| engine bug | `expect_move_object_to_zone`, `expect_move_object_to_bottom_of_zone` | `debug_assert!` on either `Err` |
| engine bug | `expect_add_object` | `debug_assert!` (its only `Err` is `ZoneNotFound`) |
| engine bug | `layers::expect_characteristics` | `debug_assert!`; printed characteristics in release |
| engine bug | `debug_assert_object_live!(state, id)` | the assert *without* a whole-struct borrow |
| legal game state | `lki_object`, `lki_object_mut` | silent `None`, CR-cited |
| legal game state | `lki_move_object_to_zone` | `ObjectNotFound` ⇒ `None`; `ZoneNotFound` still asserts |

Both families return `Option`, so converting a site is a one-token change and the
`if let Some(..)` shape survives untouched.

`lki_move_object_to_zone` is the one that earns its keep: the codebase repeatedly
wrote `Err(_) => { /* card disappeared */ }`, which is a correct fizzle for
`ObjectNotFound` and a swallowed bug for `ZoneNotFound`. Splitting the variants costs
nothing and closes that half.

## Results

398 candidate sites, every one classified.

| Verdict | Sites | Meaning |
|---|---:|---|
| IMPOSSIBLE | 222 | absence is an engine bug; now asserts |
| FIZZLE | 69 | absence is legal; documented with a CR citation |
| NONSWALLOW | 107 | nothing is discarded (see below) |
| **Total** | **398** | |

### By pattern

| Pattern | IMPOSSIBLE | FIZZLE | NONSWALLOW |
|---|---:|---:|---:|
| `OBJECTS_GET` (`state.objects.get[_mut]`) | 83 | 47 | 15 |
| `MOVE_ZONE` (`move_object_to_[bottom_of_]zone`) | 53 | 15 | 12 |
| `PLAYERS_GET` | 32 | 0 | 0 |
| `CALC_CHARS` (`calculate_characteristics`) | 26 | 5 | 58 |
| `ADD_OBJECT` | 11 | 0 | 3 |
| `ZONES_GET` | 8 | 2 | 3 |
| `LET_UNDERSCORE` + `MOVE_ZONE` | 7 | 0 | 0 |
| `OK_DISCARD` (`.ok()`) | 2 | 0 | 1 |
| `LET_UNDERSCORE` (bare) | 0 | 0 | 6 |
| `OBJECT_ACC`, `UNWRAP`, `OBJECTS_GET+UNWRAP` | 0 | 0 | 9 |

### CR citations on the fizzle branches

Per project convention and the SR-4/SR-5 gotcha, these cite the **rule text** (via the
`mtg-rules` MCP server), not card rulings.

| Rule | Sites | What it licenses |
|---|---:|---|
| CR 400.7 | 33 | zone change makes a new object; the old id is dead |
| CR 608.2b | 21 | an effect can't find info about an illegal target; that part doesn't happen |
| CR 113.7a | 11 | an ability outlives its source; LKI is used |
| CR 701.40f | 1 | Manifest from an empty library does nothing |
| CR 701.58a | 1 | Cloak from an empty library does nothing |
| CR 608.3b | 1 | a permanent spell whose target became illegal doesn't resolve (Aura attach) |
| CR 701.21a | 1 | you can't sacrifice a permanent you no longer control |

### What "NONSWALLOW" means

The sweep's regexes match some patterns that are not discarded failures. These were
classified and left alone deliberately:

- **Scalar characteristic defaults.** `power: None` is how a characteristic-defining
  `*/*` creature is represented (CR 208.2); `.unwrap_or(0)` is the correct reading of
  it, not a swallowed lookup. Same for `ctx.damaged_player.unwrap_or(ctx.controller)`,
  which selects between two valid sources.
- **`?`-propagation.** The error reaches the caller; nothing is swallowed.
- **Handled `Err` arms.** A `match` whose `Err` arm does documented work.
- **Predicates.** `.map(|o| ..).unwrap_or(false)` over a possibly-absent object is a
  question ("does a live object with this id have infect?"), and `false` is the right
  answer when it is absent.
- **`.unwrap()` guarded by a preceding `len() == 1` or present-check.**
- **`let _ = x;` silencing an intentionally-unused binding.**

## Notable findings

**The two sites the task named were real.** `combat.rs:998` and `combat.rs:1223` both
read `calculate_characteristics(state, obj.id).unwrap_or_default()` where `obj` came
straight out of `state.objects.values()`, so `None` was impossible. But *had* it
fired, the blank `Characteristics` contains no `CardType::Land`, so a landwalk check
would have silently decided the defender controls no matching land and the creature
would have become blockable. A dead branch that would have been wrong.

**36 fallbacks silently substituted pre-layer characteristics.**
`calculate_characteristics(state, id).unwrap_or_else(|| obj.characteristics.clone())`
appears throughout. The fallback returns *printed* characteristics, ignoring every
continuous effect — a Blood Moon'd land, an animated Gideon, a pumped creature would
all read wrong. Unreachable at each of these sites, but wrong if reached. All 36 are
now `expect_characteristics`, which asserts in debug and keeps the same fallback in
release.

**`move_object_to_zone`'s two error variants were collapsed.** `Err(_) => {}` treated
a missing object (legal, CR 400.7) and a missing zone (impossible) identically at
every site. Now split.

**Five sites keep raw field access.** `state.objects.get_mut(&id)` borrows one field;
`state.expect_object_mut(id)` borrows all of `GameState`. Five blocks need
`state.card_registry` or `state.timestamp_counter` through the disjoint borrow — the
exact hazard SR-3's session log warned about. Those use
`debug_assert_object_live!(state, id)` and keep the field access. The assertion is the
deliverable; the accessor is sugar.

**No assertion fired.** The full suite (3111 tests) passes with debug assertions
enabled. That is the expected and desired result: SR-4 converts a process guarantee
into a machine guarantee, it does not fix live bugs. The 222 asserts are a tripwire
for the next regression, and they are exercised by whatever coverage the suite already
has.

## Deliberately out of scope (filed as ESM tasks)

- **`scutemob-65` (SR-13)** — `effects/mod.rs` reads the *damage source's* wither and
  infect keywords via `calculate_characteristics(state, ctx.source)`, which returns
  `None` once the source has changed zones. The code then treats it as having neither.
  But CR 702.80c: "The wither rules function no matter what zone an object with wither
  deals damage from," CR 702.90e says the same for infect, and CR 113.7a requires LKI
  for a departed source. A creature with infect that dies with its damage ability on
  the stack must still deal poison. SR-4 classified these as fizzles — `None` *is* a
  legal state to observe there — and left the semantics alone rather than silently
  widening scope. The fix is an LKI snapshot, not an assertion.
- **`scutemob-66` (SR-14)** — the same vocabulary applied to the rest of `rules/`.
  Unswept `calculate_characteristics` call sites: `abilities.rs` 71, `casting.rs` 36,
  `combat.rs` 21, `sba.rs` 15, `replacement.rs` 14, `turn_actions.rs` 12, `mana.rs` 9,
  `layers.rs` 7, `engine.rs` 5, `copy.rs` 3, `continuous_effect.rs` 3, `lands.rs` 2.

## Method (reusable)

1. Enumerate candidates by pattern with a script, not by eye — `state.players.get`,
   `state.objects.get[_mut]`, `state.zones.get[_mut]`, `calculate_characteristics`,
   `move_object_to_[bottom_of_]zone`, `add_object`, `let _ =`, `.ok()`, `.unwrap()`.
   Print the count and keep the list; it is the coverage denominator.
2. Establish the ground truths above **first**. They collapse most of the work.
3. Fan out read-only classifiers over disjoint line ranges, each returning one row per
   assigned site (verdict, confidence, CR citation, one-clause rationale) and required
   to return a row for every site. Verify coverage by set difference against the
   candidate list — 398 in, 398 out.
4. Bias classifiers toward FIZZLE under uncertainty. A wrong IMPOSSIBLE panics the
   test suite (loud, cheap); a wrong FIZZLE merely fails to catch a future bug.
5. Convert mechanically, compile after each family, and **let the test suite adjudicate
   the uncertain calls**. 31 of the 222 IMPOSSIBLE verdicts were `med` confidence; the
   suite ran them and none fired.
6. Where the code's own comment contradicts a classifier ("card disappeared — nothing
   to cast" on a branch a classifier called impossible), **believe the code.** Three
   sites were demoted from IMPOSSIBLE to FIZZLE this way.

---

## Per-site classification

Line numbers are as of `ef0d9579` (the pre-change tree). `NONSWALLOW` rows are
included so the table is a complete account of the 398 candidates the sweep
considered, not just the ones it changed.

### `crates/engine/src/effects/mod.rs` — 219 sites

| Line | Pattern | Verdict | Conf | CR | Rationale |
|---:|---|---|---|---|---|
| 291 | `CALC_CHARS` | FIZZLE | med | CR 113.7a | ctx.source infect-keyword LKI lookup; damage source may have left its zone, default no-infect is a legal state |
| 297 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players are never removed; get_mut(&p) None is a broken invariant |
| 315 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players are never removed; get_mut(&p) None is a broken invariant |
| 340 | `CALC_CHARS` | IMPOSSIBLE | high | - | calc is inside state.objects.get(&id).map so id is proven present; calc None is unreachable |
| 375 | `OBJECTS_GET` | FIZZLE | med | CR 608.2b | damage target (from ctx targets) may be an illegal/gone target on resolution; do-nothing is correct |
| 390 | `CALC_CHARS` | FIZZLE | med | CR 113.7a | ctx.source wither/infect LKI lookup; source may be gone, default no-keyword is a legal state |
| 401 | `OBJECTS_GET` | FIZZLE | med | CR 608.2b | damage target may have left its zone before marking; do-nothing is correct |
| 433 | `OBJECTS_GET` | FIZZLE | med | CR 608.2b | damage target may have left its zone before marking; do-nothing is correct |
| 455 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players are never removed; get_mut(&p) None is a broken invariant |
| 478 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players are never removed; get_mut(&p) None is a broken invariant |
| 509 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players are never removed; get_mut(&p) None is a broken invariant |
| 525 | `PLAYERS_GET` | IMPOSSIBLE | high | - | ctx.controller is always a live player; get_mut None is a broken invariant |
| 604 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players are never removed; get_mut(&p) None is a broken invariant |
| 646 | `ADD_OBJECT` | IMPOSSIBLE | high | - | add_object to Battlefield only errs ZoneNotFound; battlefield always exists so Err is a bug |
| 696 | `ADD_OBJECT` | IMPOSSIBLE | high | - | add_object to Battlefield only errs ZoneNotFound; Err is a bug |
| 723 | `OBJECTS_GET` | NONSWALLOW | high | - | get is guarded by source_on_bf (equip present); and_then None is legit attached_to=None, no error swallowed |
| 725 | `OBJECTS_GET` | FIZZLE | low | CR 400.7 | defensive detach; attached_to may reference a stale/gone prev target, skip is correct |
| 733 | `OBJECTS_GET` | IMPOSSIBLE | high | - | equip_id (ctx.source) proven present via source_on_bf guard, no intervening zone move |
| 737 | `OBJECTS_GET` | IMPOSSIBLE | high | - | token_id returned by add_object earlier in same fn with no intervening move |
| 776 | `ADD_OBJECT` | IMPOSSIBLE | high | - | add_object to Battlefield only errs ZoneNotFound; Err is a bug |
| 859 | `CALC_CHARS` | IMPOSSIBLE | high | - | calc is inside state.objects.get(&id).map so id proven present; calc None unreachable |
| 903 | `MOVE_ZONE` | FIZZLE | med | CR 400.7 | target processed in a loop; object may have left zone, move ObjectNotFound is an LKI fizzle |
| 966 | `MOVE_ZONE` | FIZZLE | med | CR 400.7 | target processed in a loop; move ObjectNotFound is an LKI fizzle, do-nothing correct |
| 1011 | `CALC_CHARS` | IMPOSSIBLE | high | - | calc over live objects.iter() key **id; object present so calc None unreachable |
| 1081 | `CALC_CHARS` | IMPOSSIBLE | high | - | calc inside state.objects.get(&id).map so id proven present; calc None unreachable |
| 1121 | `MOVE_ZONE` | FIZZLE | med | CR 400.7 | id collected into Vec before mutating loop; move ObjectNotFound is an LKI fizzle |
| 1187 | `MOVE_ZONE` | FIZZLE | med | CR 400.7 | id collected into Vec before mutating loop; move ObjectNotFound is an LKI fizzle |
| 1286 | `CALC_CHARS` | IMPOSSIBLE | high | - | calc inside state.objects.get(&id).map so id proven present; calc None unreachable |
| 1326 | `MOVE_ZONE` | FIZZLE | med | CR 400.7 | target processed in a loop; move ObjectNotFound is an LKI fizzle |
| 1396 | `MOVE_ZONE` | FIZZLE | med | CR 400.7 | target processed in a loop; move ObjectNotFound is an LKI fizzle |
| 1438 | `MOVE_ZONE` | IMPOSSIBLE | high | - | old_id just verified present in graveyard via should_reanimate; move to Battlefield cannot fail |
| 1440 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id returned by move_object_to_zone Ok immediately above with no intervening move |
| 1506 | `CALC_CHARS` | IMPOSSIBLE | high | - | calc over live objects.iter() key **id; object present so calc None unreachable |
| 1532 | `CALC_CHARS` | IMPOSSIBLE | high | - | calc inside state.objects.get(&id).map so id proven present; calc None unreachable |
| 1554 | `MOVE_ZONE` | FIZZLE | med | CR 400.7 | id collected into Vec before mutating loop; move ObjectNotFound is an LKI fizzle |
| 1593 | `MOVE_ZONE` | FIZZLE | med | CR 400.7 | id collected into Vec before mutating loop; move ObjectNotFound is an LKI fizzle |
| 1629 | `CALC_CHARS` | IMPOSSIBLE | high | - | calc over live objects.iter() key **id; object present so calc None unreachable |
| 1663 | `CALC_CHARS` | IMPOSSIBLE | high | - | calc inside state.objects.get(&id).map so id proven present; calc None unreachable |
| 1685 | `MOVE_ZONE` | FIZZLE | med | CR 400.7 | id collected into Vec before mutating loop; move ObjectNotFound is an LKI fizzle |
| 1725 | `MOVE_ZONE` | FIZZLE | med | CR 400.7 | id collected into Vec before mutating loop; move ObjectNotFound is an LKI fizzle |
| 1768 | `CALC_CHARS` | IMPOSSIBLE | high | - | calc inside state.objects.get(&id).map so id proven present; calc None unreachable |
| 1798 | `MOVE_ZONE` | FIZZLE | med | CR 608.2b | ExileObject target processed in a loop; move ObjectNotFound is an LKI fizzle |
| 1842 | `MOVE_ZONE` | FIZZLE | med | CR 608.2b | ExileObject target processed in a loop; move ObjectNotFound is an LKI fizzle |
| 1913 | `MOVE_ZONE` | FIZZLE | med | CR 400.7 | source_object from stack payload; may be gone (author defaults owner too), move ObjectNotFound is an LKI fizzle |
| 1951 | `OBJECTS_GET` | FIZZLE | high | CR 608.2b | TapPermanent id is a resolved target; may have left its zone by resolution |
| 1968 | `OBJECTS_GET` | FIZZLE | high | CR 608.2b | UntapPermanent id is a resolved target; legal absence at resolution |
| 1990 | `OBJECTS_GET` | FIZZLE | high | CR 608.2b | PreventNextUntap id is a resolved target; legal absence at resolution |
| 2010 | `CALC_CHARS` | NONSWALLOW | high | - | calc_characteristics uses unwrap_or_else fallback; no error swallowed |
| 2028 | `OBJECTS_GET` | IMPOSSIBLE | high | - | ids collected from live objects; loop only flips tapped, no zone change between collect and use |
| 2056 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players never removed from state.players (ground truth 1) |
| 2073 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players never removed from state.players (ground truth 1) |
| 2093 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players never removed from state.players (ground truth 1) |
| 2119 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players never removed from state.players (ground truth 1) |
| 2150 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players never removed from state.players (ground truth 1) |
| 2171 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players never removed from state.players (ground truth 1) |
| 2195 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players never removed from state.players (ground truth 1) |
| 2212 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players never removed from state.players (ground truth 1) |
| 2247 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players never removed from state.players (ground truth 1) |
| 2284 | `OBJECTS_GET` | FIZZLE | high | CR 608.2b | AddCounter id is a resolved target; may be illegal/gone at resolution |
| 2319 | `OBJECTS_GET` | FIZZLE | high | CR 608.2b | AddCounterAmount id is a resolved target; may be gone at resolution |
| 2364 | `OBJECTS_GET` | FIZZLE | high | CR 608.2b | RemoveCounter id is a resolved target; may be gone at resolution |
| 2404 | `CALC_CHARS` | IMPOSSIBLE | high | - | id from live state.objects iteration; calc_characteristics None impossible, ? filters a dead case |
| 2430 | `OBJECTS_GET` | IMPOSSIBLE | high | - | chosen_id picked from creatures collected from live objects; no zone change before get_mut |
| 2471 | `CALC_CHARS` | IMPOSSIBLE | high | - | id from live state.objects iteration; calc_characteristics None impossible, ? filters a dead case |
| 2509 | `ADD_OBJECT` | IMPOSSIBLE | high | - | add_object errors only on ZoneNotFound; Battlefield always exists (ground truth 6) |
| 2563 | `OBJECTS_GET` | IMPOSSIBLE | high | - | army_id created/chosen in this fn; apply_counter_replacement does not move objects |
| 2585 | `OBJECTS_GET` | IMPOSSIBLE | high | - | army_id valid; only a counter insert occurred between prior get and this get_mut |
| 2617 | `CALC_CHARS` | NONSWALLOW | high | - | calc_characteristics feeds and_then().or(fallback); id pre-filtered present, no swallow |
| 2624 | `MOVE_ZONE` | FIZZLE | high | CR 400.7 | MoveZone id is a resolved target; ObjectNotFound is a legitimate LKI fizzle |
| 2631 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id just returned Ok by move_object_to_zone above; no intervening move |
| 2716 | `MOVE_ZONE` | IMPOSSIBLE | med | - | ids collected this resolution from from_zone; disjoint, move_object_to_zone runs no SBAs |
| 2775 | `OBJECTS_GET` | NONSWALLOW | high | - | if-let-Some else is handled (returns false in retain); id from live collect, no swallow |
| 2797 | `ZONES_GET` | IMPOSSIBLE | high | - | zones never lose entries; Library(p) always present (ground truth 2) |
| 2807 | `MOVE_ZONE` | IMPOSSIBLE | med | - | card_id found live this resolution; shuffle reorders not removes; Battlefield exists |
| 2810 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id just returned Ok by move_object_to_zone above; no intervening move |
| 2821 | `LET_UNDERSCORE+MOVE_ZONE` | IMPOSSIBLE | med | - | card_id found live this resolution; hand/graveyard dest exists, no removal between find and move |
| 2848 | `LET_UNDERSCORE+MOVE_ZONE` | IMPOSSIBLE | med | - | Scry ids from live library zone; disjoint, moving each within library, no SBAs |
| 2883 | `LET_UNDERSCORE+MOVE_ZONE` | IMPOSSIBLE | med | - | Surveil ids from live library zone; disjoint, library->graveyard, no SBAs |
| 2900 | `ZONES_GET` | IMPOSSIBLE | high | - | zones never lose entries; Library(p) always present (ground truth 2) |
| 3243 | `PLAYERS_GET` | IMPOSSIBLE | high | - | pid collected from live players; players never removed (ground truth 1) |
| 3262 | `OBJECTS_GET` | FIZZLE | high | CR 608.2b | Goad id is a resolved target; may be gone at resolution |
| 3281 | `OBJECTS_GET` | FIZZLE | high | CR 608.2b | Suspect id is a resolved target; may be gone at resolution |
| 3301 | `OBJECTS_GET` | FIZZLE | high | CR 608.2b | Unsuspect id is a resolved target; may be gone at resolution |
| 3336 | `CALC_CHARS` | NONSWALLOW | high | - | calc_characteristics uses unwrap_or_else fallback; obj.id from live values(), no swallow |
| 3352 | `OBJECTS_GET` | FIZZLE | med | CR 113.7a | ctx.source may not be on battlefield (spell-level use); ctx set regardless, absence legal |
| 3462 | `OBJECTS_GET` | IMPOSSIBLE | high | - | obj_id from live battlefield collect; counter replacement/insert moves nothing |
| 3491 | `PLAYERS_GET` | IMPOSSIBLE | high | - | pid collected from live players; players never removed (ground truth 1) |
| 3546 | `LET_UNDERSCORE` | NONSWALLOW | high | - | let _ = index silences an intentionally-unused match binding; nothing swallowed |
| 3566 | `ZONES_GET` | FIZZLE | high | CR 701.40f | zones.get(Library) always exists; None is z.top() on an empty library — Manifest legally does nothing |
| 3568 | `MOVE_ZONE` | IMPOSSIBLE | high | - | top_id just read from library top, no intervening move, Battlefield always exists → move cannot Err |
| 3571 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id was just returned by move_object_to_zone with no intervening move |
| 3600 | `LET_UNDERSCORE` | NONSWALLOW | high | - | let _ = index silences an intentionally-unused destructured field |
| 3616 | `ZONES_GET` | FIZZLE | high | CR 701.58a | zones.get(Library) always exists; None is z.top() on an empty library — Cloak legally does nothing |
| 3618 | `MOVE_ZONE` | IMPOSSIBLE | high | - | top_id just read from library top, Battlefield always exists → move cannot Err |
| 3621 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id was just returned by move_object_to_zone with no intervening move |
| 3692 | `OBJECTS_GET` | FIZZLE | med | CR 113.7a | ctx.source of a dies-trigger (Locust God) may already be gone; get_mut None uses last-known-info |
| 3793 | `MOVE_ZONE` | IMPOSSIBLE | high | - | source_valid confirmed source_id on battlefield this block; Exile always exists → move cannot Err |
| 3795 | `OK_DISCARD` | IMPOSSIBLE | high | - | .ok() discards an Err that cannot occur (source present, Exile exists) |
| 3797 | `MOVE_ZONE` | IMPOSSIBLE | high | - | partner_obj_id found on battlefield via values().find; Exile always exists → move cannot Err |
| 3799 | `OK_DISCARD` | IMPOSSIBLE | high | - | .ok() discards an Err that cannot occur (partner present, Exile exists) |
| 3895 | `ZONES_GET` | IMPOSSIBLE | high | - | zones.get_mut(Exile) — the Exile zone is builder-populated and never removed |
| 3944 | `OBJECTS_GET` | IMPOSSIBLE | high | - | card_id just found via objects.iter() in Exile with only reads intervening → still present |
| 3951 | `MOVE_ZONE` | IMPOSSIBLE | high | - | card_id present in Exile, Battlefield always exists; the Err(_) arm is dead |
| 3953 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id was just returned by move_object_to_zone |
| 3984 | `MOVE_ZONE` | IMPOSSIBLE | high | - | card_id present in Exile, Battlefield always exists → move cannot Err |
| 3986 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id was just returned by move_object_to_zone |
| 3998 | `MOVE_ZONE` | IMPOSSIBLE | high | - | card_id present in Exile, Graveyard always exists → move cannot Err |
| 4058 | `CALC_CHARS` | IMPOSSIBLE | med | - | target_id came from resolve_effect_target_list which filters to objects present → calc_chars is total, false-default unreachable |
| 4085 | `OBJECTS_GET` | NONSWALLOW | high | - | reads the attached_to Option field off equip_id (resolved+present); None means "not attached", a legit value |
| 4087 | `OBJECTS_GET` | FIZZLE | high | CR 400.7 | prev_target is the previously-attached creature, which may have changed zones and become a new object |
| 4095 | `OBJECTS_GET` | IMPOSSIBLE | high | - | equip_id is the resolved-present equipment source, no intervening removal in this loop |
| 4099 | `OBJECTS_GET` | IMPOSSIBLE | high | - | target_id is resolved-present, no intervening zone change since resolution |
| 4113 | `CALC_CHARS` | IMPOSSIBLE | high | - | equip_id present (just attached); calc_chars is total → false-default unreachable |
| 4121 | `OBJECTS_GET` | IMPOSSIBLE | high | - | equip_id is the resolved-present equipment source |
| 4164 | `CALC_CHARS` | IMPOSSIBLE | med | - | equip_id (fortification) came from resolve_effect_target_list which filters to present → calc_chars total |
| 4196 | `CALC_CHARS` | IMPOSSIBLE | med | - | target_id came from resolve_effect_target_list which filters to present → calc_chars total |
| 4224 | `OBJECTS_GET` | NONSWALLOW | high | - | reads the attached_to Option field off equip_id (resolved+present); None means "not attached" |
| 4226 | `OBJECTS_GET` | FIZZLE | high | CR 400.7 | prev_target is the previously-fortified land, which may have changed zones and become a new object |
| 4234 | `OBJECTS_GET` | IMPOSSIBLE | high | - | equip_id is the resolved-present fortification source |
| 4238 | `OBJECTS_GET` | IMPOSSIBLE | high | - | target_id is resolved-present, no intervening zone change |
| 4277 | `OBJECTS_GET` | NONSWALLOW | high | - | reads attached_to Option off equip_id (on_battlefield checked); None handled by let-else continue |
| 4283 | `OBJECTS_GET` | IMPOSSIBLE | high | - | equip_id confirmed on_battlefield immediately above, no intervening removal |
| 4289 | `OBJECTS_GET` | FIZZLE | med | CR 400.7 | target_id read from stored attached_to; the attached permanent may have left and become a new object |
| 4348 | `OBJECTS_GET` | NONSWALLOW | high | - | reads the card_id Option field off a present hand object; None is a tokenless object, a legit value |
| 4363 | `MOVE_ZONE` | IMPOSSIBLE | high | - | card_id just selected from hand via min_by_key, destination zone always exists → move cannot Err |
| 4408 | `OBJECTS_GET` | IMPOSSIBLE | high | - | creature_on_battlefield just confirmed creature_id present on battlefield with no intervening change |
| 4556 | `OBJECTS_GET` | IMPOSSIBLE | high | - | id collected from library zone.object_ids() immediately above with no intervening move |
| 4573 | `MOVE_ZONE` | IMPOSSIBLE | high | - | id still in library (disjoint per-id moves), resolved dest zone always exists → move cannot Err |
| 4575 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id was just returned by move_object_to_zone |
| 4586 | `MOVE_ZONE` | IMPOSSIBLE | high | - | id still in library, resolved dest zone always exists → move cannot Err |
| 4759 | `ADD_OBJECT` | IMPOSSIBLE | high | - | add_object errs only ZoneNotFound; Battlefield always exists, so Err(_)=>continue is dead |
| 4929 | `ADD_OBJECT` | IMPOSSIBLE | high | - | add_object errs only ZoneNotFound; Command(ctrl) always exists, so Err(_)=>return is dead |
| 4991 | `CALC_CHARS` | NONSWALLOW | high | - | calc None caught by .or(o.characteristics.power) fallback; obj already verified present |
| 4998 | `MOVE_ZONE` | IMPOSSIBLE | high | - | id re-verified on_bf same iteration with no intervening mutation; Exile zone always exists |
| 5009 | `MOVE_ZONE` | IMPOSSIBLE | high | - | exile_id returned by move_object_to_zone just above, no intervening move; Battlefield exists |
| 5011 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_bf_id returned by move_object_to_zone two lines up, no intervening move |
| 5077 | `CALC_CHARS` | NONSWALLOW | high | - | calc None caught by .or(o.characteristics.power) fallback; obj bound in Some(o) arm |
| 5089 | `MOVE_ZONE` | IMPOSSIBLE | high | - | reaching here means owner was Some (obj verified on_bf); no intervening mutation; Exile exists |
| 5127 | `PLAYERS_GET` | IMPOSSIBLE | high | - | controller=ctx.controller; players never removed |
| 5178 | `OBJECTS_GET` | FIZZLE | med | CR 608.2b | obj_id from resolved target; target may have left before resolution, get_mut None skips control update |
| 5207 | `OBJECTS_GET` | FIZZLE | med | CR 608.2b | a_id from resolved target; may have left, None flows to (Some,Some) guard = do nothing |
| 5208 | `OBJECTS_GET` | FIZZLE | med | CR 608.2b | b_id from resolved target; may have left, None flows to (Some,Some) guard = do nothing |
| 5229 | `OBJECTS_GET` | IMPOSSIBLE | med | - | obj_id is a_id/b_id, both verified present via a_ctrl/b_ctrl Some at 5207-5209; no move since |
| 5249 | `PLAYERS_GET` | IMPOSSIBLE | high | - | pid from resolve_player_target_list; players never removed |
| 5296 | `MOVE_ZONE` | IMPOSSIBLE | high | - | land_id freshly scanned from state.objects this line, no loop/mutation; Battlefield exists |
| 5300 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_land_id returned by move_object_to_zone just above, no intervening move |
| 5358 | `OBJECTS_GET` | FIZZLE | med | CR 113.7a | ctx.source is LKI; SolveCase source may have left battlefield, doing nothing is correct |
| 5532 | `MOVE_ZONE` | FIZZLE | med | CR 400.7 | old_id collected before loop that runs ETB replacements which can move other candidates |
| 5534 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id returned by move_object_to_zone just above, no intervening move |
| 5621 | `MOVE_ZONE` | IMPOSSIBLE | med | - | old_id from graveyard snapshot; step1 loop only exiles (no triggers process mid-loop); Exile exists |
| 5666 | `OBJECTS_GET` | FIZZLE | high | CR 400.7 | explicit "already gone (moved by earlier step)" — id from pre-loop battlefield snapshot |
| 5669 | `CALC_CHARS` | NONSWALLOW | high | - | calc None caught by unwrap_or_else(obj.characteristics.clone()); obj bound in Some arm |
| 5702 | `MOVE_ZONE` | IMPOSSIBLE | med | - | id verified present at 5666; check_zone_change_replacement does not move it; dest zones always exist |
| 5775 | `MOVE_ZONE` | IMPOSSIBLE | med | - | id verified present at 5666, no intervening move; Graveyard(owner) always exists |
| 5822 | `MOVE_ZONE` | IMPOSSIBLE | high | - | exile_id verified still_in_exile immediately above; Battlefield always exists |
| 5825 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id returned by move_object_to_zone just above, no intervening move |
| 6088 | `PLAYERS_GET` | IMPOSSIBLE | high | - | get None impossible (players never removed); the has_lost=empty path is legal target-invalidity |
| 6140 | `CALC_CHARS` | NONSWALLOW | high | - | calc None caught by unwrap_or_else fallback; id from live state.objects iteration |
| 6161 | `CALC_CHARS` | NONSWALLOW | high | - | calc None caught by unwrap_or_else fallback; id from live state.objects iteration |
| 6209 | `OBJECTS_GET` | FIZZLE | med | CR 113.7a | ctx.source is LKI; equipment source may have left, None yields empty target list |
| 6279 | `OBJECTS_GET` | NONSWALLOW | med | - | objects.get None explicitly falls through to stack_objects lookup (ward-on-stack case) |
| 6302 | `OBJECTS_GET` | FIZZLE | med | CR 400.7 | id from resolved target; if gone, filter_map drops it (owner-bounce fizzle) |
| 6369 | `OBJECT_ACC` | NONSWALLOW | high | - | .object(source).ok() -> None handled by both Some/None match arms downstream |
| 6370 | `OK_DISCARD` | NONSWALLOW | high | - | .ok() converts error to Option that is fully handled by the following match |
| 6373 | `UNWRAP` | NONSWALLOW | high | - | restriction guaranteed Some by outer match arm (ChosenType* variants); unwrap infallible |
| 6383 | `UNWRAP` | NONSWALLOW | high | - | restriction guaranteed Some by outer match arm; unwrap infallible |
| 6421 | `CALC_CHARS` | NONSWALLOW | high | - | id verified on_bf (returns false earlier if absent); unwrap_or(false) default; calc will be Some |
| 6428 | `CALC_CHARS` | NONSWALLOW | high | - | documented scalar default unwrap_or(0) for absent/no-power object |
| 6475 | `CALC_CHARS` | NONSWALLOW | high | - | keyword checks use .map().unwrap_or(false); source present within fn, defensive default only |
| 6493 | `OBJECTS_GET` | IMPOSSIBLE | med | - | target validated on-battlefield by Fight/Bite caller immediately before; no intervening zone change |
| 6528 | `OBJECTS_GET` | IMPOSSIBLE | med | - | source present: lifelink=true implies calc returned Some at 6475; no move since |
| 6529 | `PLAYERS_GET` | IMPOSSIBLE | high | - | controller_id read from live object; players never removed |
| 6556 | `OBJECTS_GET` | NONSWALLOW | med | - | resolve list pre-filters to existing objects; scalar unwrap_or(0) default on empty |
| 6558 | `CALC_CHARS` | NONSWALLOW | high | - | calc None caught by unwrap_or_else fallback; obj bound in and_then |
| 6579 | `OBJECTS_GET` | NONSWALLOW | med | - | resolve list pre-filters to existing objects; scalar unwrap_or(0) default on empty |
| 6581 | `CALC_CHARS` | NONSWALLOW | high | - | calc None caught by unwrap_or_else fallback; obj bound in and_then |
| 6601 | `OBJECTS_GET` | FIZZLE | high | CR 608.2b | ManaValueOf reads a resolved target id; illegal/gone target yields 0 via filter_map→next→unwrap_or(0) |
| 6644 | `CALC_CHARS` | NONSWALLOW | high | - | obj.id from state.objects.values() iteration; None handled by unwrap_or_else base-chars fallback |
| 6672 | `CALC_CHARS` | NONSWALLOW | high | - | obj.id from values() iteration; unwrap_or_else fallback to base characteristics, nothing discarded |
| 6811 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players.get(pid) in filter_map; PlayerState never removed so None is a bug |
| 6853 | `CALC_CHARS` | NONSWALLOW | high | - | obj.id from values() iteration; unwrap_or_else base-chars fallback |
| 6882 | `CALC_CHARS` | NONSWALLOW | high | - | obj.id from values() iteration; unwrap_or_else base-chars fallback |
| 6942 | `CALC_CHARS` | NONSWALLOW | high | - | obj.id from values() iteration; unwrap_or_else base-chars fallback |
| 6973 | `CALC_CHARS` | NONSWALLOW | high | - | obj.id from values() iteration; unwrap_or_else base-chars fallback |
| 7283 | `CALC_CHARS` | NONSWALLOW | high | - | **id from objects.iter() filter; unwrap_or_else base-chars fallback |
| 7351 | `OBJECTS_GET` | FIZZLE | high | CR 400.7 | id from to_sacrifice collected before the sacrificing loop; None=>continue is a legitimate LKI fizzle |
| 7353 | `CALC_CHARS` | NONSWALLOW | high | - | id just confirmed live via Some(obj); result Option clone().unwrap_or_else handled |
| 7387 | `MOVE_ZONE` | IMPOSSIBLE | high | - | move of live id (Some(obj) at 7351) to a valid Redirect ZoneId; Err cannot occur |
| 7459 | `MOVE_ZONE` | IMPOSSIBLE | high | - | move of live id (Some(obj) at 7351) to Graveyard(owner) valid zone; Err cannot occur |
| 7569 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players.get_mut(pid) if-let; PlayerState never removed so None is a bug |
| 7579 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players.get_mut(pid) if-let; PlayerState never removed so None is a bug |
| 7650 | `ZONES_GET` | NONSWALLOW | high | - | zones.get never None (zones persist); only z.top() None (empty library) which is handled as loss at 7652 |
| 7654 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players.get_mut(player) to set has_lost; PlayerState never removed so None is a bug |
| 7663 | `MOVE_ZONE` | IMPOSSIBLE | high | - | move of card_id from live library top to Hand(player) valid zone; else vec![] swallows impossible Err |
| 7665 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players.get_mut(player) cards_drawn increment; PlayerState never removed so None is a bug |
| 7703 | `OBJECTS_GET` | NONSWALLOW | high | - | card_id just found via min_by_key (live); o.card_id None is a legit token state, consumed downstream |
| 7718 | `MOVE_ZONE` | IMPOSSIBLE | high | - | move of live hand card_id to valid destination zone; if-let swallows impossible Err |
| 7776 | `LET_UNDERSCORE+MOVE_ZONE` | IMPOSSIBLE | med | - | ids collected before loop; each distinct id moved exactly once, no id invalidated by another's move, so let _ discards an impossible Err |
| 7781 | `ZONES_GET` | IMPOSSIBLE | high | - | zones.get_mut(lib_zone) if-let; library zone always exists so None (skip shuffle) is a bug |
| 7791 | `ZONES_GET` | NONSWALLOW | high | - | zones.get never None; z.top() None (empty library) correctly skips mill |
| 7793 | `MOVE_ZONE` | IMPOSSIBLE | high | - | move of live library top to Graveyard(player) valid zone; if-let swallows impossible Err |
| 8004 | `CALC_CHARS` | NONSWALLOW | high | - | obj.id from values() iteration; unwrap_or_else base-chars fallback |
| 8016 | `CALC_CHARS` | NONSWALLOW | high | - | obj.id from values() iteration; unwrap_or_else base-chars fallback |
| 8037 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players.get(p) for target player .unwrap_or(false); PlayerState never removed so None is a bug |
| 8112 | `CALC_CHARS` | NONSWALLOW | high | - | obj.id from values() iteration; unwrap_or_else base-chars fallback |
| 8129 | `CALC_CHARS` | NONSWALLOW | high | - | id from objects.iter() filter; unwrap_or_else base-chars fallback |
| 8170 | `CALC_CHARS` | NONSWALLOW | high | - | obj.id from values() iteration; unwrap_or_else base-chars fallback |
| 8192 | `CALC_CHARS` | NONSWALLOW | high | - | id from objects.iter() filter; unwrap_or_else base-chars fallback |
| 8211 | `CALC_CHARS` | NONSWALLOW | high | - | id from objects.iter() filter; unwrap_or_else base-chars fallback |
| 8225 | `CALC_CHARS` | NONSWALLOW | high | - | obj.id from values() iteration; unwrap_or_else base-chars fallback |
| 8237 | `CALC_CHARS` | NONSWALLOW | high | - | obj.id from values() iteration; unwrap_or_else base-chars fallback |
| 8316 | `ZONES_GET` | NONSWALLOW | high | - | zones.get never None; z.top() None (empty library) handled by match _=>false |
| 8319 | `OBJECTS_GET` | IMPOSSIBLE | high | - | id from z.top() read immediately above in read-only check; objects.get None cannot occur, .unwrap_or(false) swallows a bug |
| 8383 | `CALC_CHARS` | NONSWALLOW | high | - | o.id from values() iteration; unwrap_or_else base-chars fallback |
| 8452 | `CALC_CHARS` | NONSWALLOW | high | - | obj.id from values() iteration; unwrap_or_else base-chars fallback |
| 8602 | `CALC_CHARS` | NONSWALLOW | high | - | **id from objects.iter() filter; unwrap_or_else base-chars fallback |
| 8616 | `CALC_CHARS` | NONSWALLOW | high | - | **id from objects.iter() filter; unwrap_or_else base-chars fallback |
| 8630 | `CALC_CHARS` | NONSWALLOW | high | - | **id from objects.iter() filter; unwrap_or_else base-chars fallback |
| 8647 | `CALC_CHARS` | NONSWALLOW | high | - | **id from objects.iter() filter; unwrap_or_else base-chars fallback |

### `crates/engine/src/rules/resolution.rs` — 179 sites

| Line | Pattern | Verdict | Conf | CR | Rationale |
|---:|---|---|---|---|---|
| 56 | `OBJECTS_GET` | IMPOSSIBLE | med | CR 400.7 | source_object is the currently-resolving spell's own card, still on the stack in state.objects, so absence would be a bug |
| 85 | `OBJECT_ACC` | NONSWALLOW | high | - | state.object(source_object)? propagates the GameStateError, nothing swallowed |
| 99 | `MOVE_ZONE` | NONSWALLOW | high | - | move_object_to_zone(..)? propagates the error, nothing swallowed |
| 127 | `OBJECT_ACC` | NONSWALLOW | high | - | state.object(source_object)? propagates the error, nothing swallowed |
| 544 | `PLAYERS_GET` | IMPOSSIBLE | high | - | controller PlayerId; GameState.players never removes entries so get_mut None is a bug |
| 576 | `MOVE_ZONE` | NONSWALLOW | high | - | move_object_to_zone(..)? propagates the error, nothing swallowed |
| 583 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id returned by move_object_to_zone just above with no intervening move |
| 831 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id is the just-entered battlefield object, still present |
| 854 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id is the just-entered battlefield object, still present |
| 900 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id is the just-entered battlefield object, still present |
| 942 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id is the just-entered battlefield object, still present |
| 979 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id is the just-entered battlefield object, still present |
| 1027 | `CALC_CHARS` | IMPOSSIBLE | high | - | calculate_characteristics on new_id which is present on the battlefield |
| 1054 | `CALC_CHARS` | IMPOSSIBLE | high | - | hand_id read from objects iteration just above with no intervening zone change |
| 1071 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id is the just-entered battlefield object, still present |
| 1129 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id is the just-entered battlefield object, still present |
| 1226 | `OBJECTS_GET` | FIZZLE | med | CR 608.3b | sac_id is a sacrifice cost id re-validated at resolution; a prior loop iteration may have moved it |
| 1233 | `CALC_CHARS` | IMPOSSIBLE | high | - | calculate_characteristics on sac_id already confirmed present by the Some(o) match above |
| 1263 | `MOVE_ZONE` | IMPOSSIBLE | high | - | sac_id validated present and dest is a real zone; only ZoneNotFound (always a bug) could Err |
| 1302 | `MOVE_ZONE` | IMPOSSIBLE | high | - | sac_id present and Graveyard(sac_owner) always exists; an Err would be a bug |
| 1328 | `MOVE_ZONE` | IMPOSSIBLE | high | - | sac_id present and Graveyard(sac_owner) always exists; an Err would be a bug |
| 1353 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id is the just-entered battlefield object, still present |
| 1362 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id is the just-entered battlefield object, still present |
| 1417 | `LET_UNDERSCORE` | NONSWALLOW | high | - | let _ = tribute_instances silences an unused binding, no lookup or error |
| 1440 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id is the just-entered battlefield object, still present |
| 1502 | `CALC_CHARS` | IMPOSSIBLE | high | - | calculate_characteristics on new_id which is present on the battlefield |
| 1533 | `CALC_CHARS` | IMPOSSIBLE | high | - | calculate_characteristics on new_id which is present on the battlefield |
| 1584 | `CALC_CHARS` | IMPOSSIBLE | high | - | calculate_characteristics on new_id which is present on the battlefield |
| 1595 | `OBJECTS_GET` | IMPOSSIBLE | high | - | objects.get(new_id) present; the and_then None only reflects a legit gift_opponent None |
| 1622 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id is the just-entered battlefield object, still present |
| 1639 | `OBJECTS_GET` | IMPOSSIBLE | high | - | objects.get(new_id) present on the battlefield |
| 1665 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id (the aura) is the just-entered battlefield object, still present |
| 1669 | `OBJECTS_GET` | FIZZLE | med | CR 608.2b | target_id is the aura's target from the stack; if it became illegal the aura is left unattached |
| 1801 | `CALC_CHARS` | IMPOSSIBLE | high | - | calculate_characteristics on obj.id currently being iterated from state.objects |
| 1850 | `MOVE_ZONE` | NONSWALLOW | high | - | move_object_to_zone(..)? propagates the error, nothing swallowed |
| 1854 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id returned by move_object_to_zone at line 1850 with no intervening move |
| 1865 | `OBJECTS_GET` | FIZZLE | med | CR 400.7 | cipher_creature is a chosen/target creature that may have left the battlefield before encode; skipping is legal |
| 1979 | `OBJECTS_GET` | FIZZLE | high | CR 113.7a | triggered-ability source may have left its zone; None falls to (None,None) and ability resolves as LKI with no effect |
| 1985 | `CALC_CHARS` | NONSWALLOW | high | - | calc_characteristics is inside `if let Some(obj)` with unwrap_or_else fallback to obj.characteristics |
| 2084 | `UNWRAP` | NONSWALLOW | high | - | unwrap is guarded by the preceding chosen_effects.len()==1 branch, cannot panic |
| 2175 | `OBJECTS_GET` | FIZZLE | high | CR 113.7a | triggered source may be gone; `None => true` resolves the ability without effect (LKI) |
| 2180 | `CALC_CHARS` | NONSWALLOW | high | - | calc_characteristics inside `Some(obj) =>` with unwrap_or_else fallback |
| 2217 | `CALC_CHARS` | NONSWALLOW | high | - | calc_characteristics inside `.and_then(\|obj\|)` with unwrap_or_else fallback |
| 2427 | `OBJECTS_GET` | IMPOSSIBLE | high | - | get_mut re-looks-up id after current_counters was Some at 2419 with no intervening zone change |
| 2514 | `OBJECTS_GET` | FIZZLE | high | CR 400.7 | vanishing permanent id from stored trigger payload; may have left battlefield, do-nothing is correct |
| 2519 | `CALC_CHARS` | NONSWALLOW | high | - | calc_characteristics result stored as Option and consumed via as_ref()/.or() fallback, not swallowed |
| 2561 | `MOVE_ZONE` | IMPOSSIBLE | med | - | object confirmed on battlefield at 2514, no intervening move; redirect dest is a valid zone so Err is a bug |
| 2593 | `MOVE_ZONE` | IMPOSSIBLE | high | - | object confirmed on battlefield, Graveyard zone always exists; move cannot legitimately fail |
| 2654 | `OBJECTS_GET` | FIZZLE | high | CR 400.7 | fading permanent id from stored trigger payload; may have left battlefield |
| 2657 | `CALC_CHARS` | NONSWALLOW | high | - | calc_characteristics stored as Option with .or() fallback, not swallowed |
| 2685 | `OBJECTS_GET` | IMPOSSIBLE | high | - | get_mut re-looks-up id after source_info confirmed present at 2654 with no intervening move |
| 2717 | `MOVE_ZONE` | IMPOSSIBLE | med | - | object confirmed on battlefield, redirect dest is a valid zone; Err is a bug |
| 2747 | `MOVE_ZONE` | IMPOSSIBLE | high | - | object confirmed on battlefield, Graveyard zone always exists |
| 2864 | `OBJECTS_GET` | IMPOSSIBLE | high | - | get_mut guarded by still_on_battlefield computed at 2857, no intervening move |
| 2954 | `OBJECTS_GET` | FIZZLE | high | CR 400.7 | evoke source may have left battlefield (blinked/bounced); do-nothing is correct |
| 2959 | `CALC_CHARS` | NONSWALLOW | high | - | calc_characteristics stored as Option with .or() fallback |
| 3000 | `MOVE_ZONE` | IMPOSSIBLE | med | - | object confirmed on battlefield at 2954, no intervening move; redirect dest valid zone |
| 3029 | `MOVE_ZONE` | IMPOSSIBLE | high | - | object confirmed on battlefield, Graveyard zone always exists |
| 3096 | `MOVE_ZONE` | IMPOSSIBLE | high | - | move guarded by still_in_exile computed at 3088, Graveyard zone exists; Err is a bug |
| 3154 | `MOVE_ZONE` | NONSWALLOW | high | - | error propagated with `?`, not swallowed |
| 3160 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id was just returned by move_object_to_zone at 3153 with no intervening move |
| 3161 | `OBJECTS_GET` | IMPOSSIBLE | high | - | get_mut on new_id just returned by move at 3153, no intervening move |
| 3235 | `CALC_CHARS` | NONSWALLOW | high | - | calc_characteristics chained with .and_then/.or fallback, not swallowed |
| 3245 | `MOVE_ZONE` | NONSWALLOW | high | - | error propagated with `?` |
| 3279 | `CALC_CHARS` | NONSWALLOW | high | - | calc_characteristics with .and_then/.or fallback |
| 3287 | `MOVE_ZONE` | NONSWALLOW | high | - | error propagated with `?` |
| 3321 | `OBJECTS_GET` | FIZZLE | high | CR 400.7 | blitz source may have left battlefield before delayed trigger resolves |
| 3326 | `CALC_CHARS` | NONSWALLOW | high | - | calc_characteristics stored as Option with .or() fallback |
| 3368 | `MOVE_ZONE` | IMPOSSIBLE | med | - | object confirmed on battlefield at 3321, no intervening move; redirect dest valid zone |
| 3397 | `MOVE_ZONE` | IMPOSSIBLE | high | - | object confirmed on battlefield, Graveyard zone always exists |
| 3457 | `CALC_CHARS` | NONSWALLOW | high | - | calc_characteristics with .and_then/.or fallback |
| 3464 | `MOVE_ZONE` | NONSWALLOW | high | - | error propagated with `?` |
| 3468 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_exile_id returned by move_object_to_zone at 3464 with no intervening move |
| 3515 | `OBJECTS_GET` | IMPOSSIBLE | high | - | object confirmed present via current_counters read at 3504, no intervening mutation |
| 3646 | `OBJECTS_GET` | IMPOSSIBLE | high | - | target_valid confirmed present-on-bf at 3640, re-lookup with no intervening move |
| 3649 | `OBJECTS_GET` | IMPOSSIBLE | high | - | inside the 3646 present block, no intervening move |
| 3685 | `OBJECTS_GET` | IMPOSSIBLE | high | - | should_resolve guard read source_object present at 3676 |
| 3774 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players are never removed from GameState, get_mut None is a bug |
| 3801 | `CALC_CHARS` | FIZZLE | med | CR 400.7 | enlisted is an LKI trigger payload that may have left, absent gives 0 bonus legally |
| 3879 | `OBJECTS_GET` | FIZZLE | high | CR 608.2b | explicit Modular target fizzle check, target may have left battlefield |
| 3881 | `CALC_CHARS` | NONSWALLOW | high | - | inside is_some_and where obj is present, unwrap_or_else fallback is dead |
| 3897 | `OBJECTS_GET` | IMPOSSIBLE | high | - | target_id validated still_legal/present at 3879, no intervening move |
| 3939 | `CALC_CHARS` | FIZZLE | high | CR 400.7 | evolve entering_creature LKI, or_else fallback handles absence -> condition false |
| 3948 | `CALC_CHARS` | FIZZLE | high | CR 113.7a | evolve source LKI, or_else fallback handles absence -> condition false |
| 3973 | `OBJECTS_GET` | FIZZLE | med | CR 113.7a | evolve source may have left after LKI condition check, inner zone guard confirms |
| 4029 | `OBJECTS_GET` | IMPOSSIBLE | high | - | source_has_counter validated present-on-bf at 4007, no intervening move |
| 4049 | `OBJECTS_GET` | IMPOSSIBLE | high | - | target_on_battlefield validated present at 4021, source mutation did not move it |
| 4115 | `CALC_CHARS` | NONSWALLOW | high | - | inside .get().map where obj present, unwrap_or(false) fallback is dead |
| 4125 | `OBJECTS_GET` | IMPOSSIBLE | high | - | target_valid validated present-on-bf at 4107, no intervening move |
| 4191 | `CALC_CHARS` | NONSWALLOW | high | - | id drawn from live objects iteration, calc always Some, else branch dead |
| 4201 | `OBJECTS_GET` | NONSWALLOW | high | - | `?` propagated inside filter_map, id from iteration is present |
| 4227 | `CALC_CHARS` | NONSWALLOW | high | - | inside .get().map where present, .or fallback for LKI power capture |
| 4234 | `MOVE_ZONE` | IMPOSSIBLE | high | - | target from live candidate filter, present, Exile zone always exists |
| 4237 | `OBJECTS_GET` | IMPOSSIBLE | high | - | champion source on-bf at 4165, only target was exiled, source untouched |
| 4247 | `LET_UNDERSCORE` | NONSWALLOW | high | - | let _ suppresses unused-variable warning, nothing swallowed |
| 4255 | `OBJECTS_GET` | FIZZLE | med | CR 701.21a | None means champion cant-be-sacrificed (or not on bf), a legal no-op |
| 4259 | `CALC_CHARS` | NONSWALLOW | high | - | inside and_then where obj present, .or fallback for LKI power |
| 4303 | `MOVE_ZONE` | IMPOSSIBLE | high | - | source present from source_info guard, redirect dest is a valid zone |
| 4338 | `MOVE_ZONE` | IMPOSSIBLE | high | - | source present from source_info guard, Graveyard(owner) zone always exists |
| 4395 | `OBJECTS_GET` | FIZZLE | high | CR 400.7 | champion exiled_card LKI, may have left exile, and_then handles absence |
| 4405 | `MOVE_ZONE` | IMPOSSIBLE | high | - | exiled_card confirmed in Exile at 4395, Battlefield zone always exists |
| 4408 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id returned by move_object_to_zone at 4404 with no intervening move |
| 4480 | `CALC_CHARS` | NONSWALLOW | med | - | && short-circuit guards calc so it only runs when source present, fallback dead |
| 4494 | `CALC_CHARS` | NONSWALLOW | med | - | && short-circuit guards calc so it only runs when pair_target present, fallback dead |
| 4499 | `OBJECTS_GET` | IMPOSSIBLE | high | - | source_ok validated source present-on-bf at 4470, no intervening move |
| 4502 | `OBJECTS_GET` | IMPOSSIBLE | high | - | target_ok validated pair_target present at 4483, 4499 mutation did not move it |
| 4737 | `ADD_OBJECT` | IMPOSSIBLE | high | - | add_object to Battlefield errs only ZoneNotFound, zone always exists; continue swallows a bug |
| 4800 | `CALC_CHARS` | NONSWALLOW | high | - | inside `if source_on_battlefield` present guard, unwrap_or(false) fallback dead |
| 4948 | `ADD_OBJECT` | IMPOSSIBLE | high | - | add_object errs only ZoneNotFound(Battlefield) which never happens; Err=>continue silently skips token creation |
| 5050 | `CALC_CHARS` | IMPOSSIBLE | high | - | inside if source_on_battlefield so object present; calc_chars None only if absent — contradiction, unwrap_or(false) masks bug |
| 5104 | `OBJECTS_GET` | FIZZLE | high | CR 400.7 | source_object is Mount from stack payload; may have left battlefield between activation and resolution — do-nothing correct |
| 5165 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players never removed; stack controller always present |
| 5235 | `MOVE_ZONE` | NONSWALLOW | high | - | error propagated with `?`; also guarded present in graveyard so cannot fail here |
| 5238 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_exile_id returned by move_object_to_zone Ok immediately above, no intervening move |
| 5324 | `OBJECTS_GET` | FIZZLE | med | CR 400.7 | haunt_source re-looked-up after execute_effect ran; intervening effect could move it — skip-clear is harmless |
| 5367 | `CALC_CHARS` | IMPOSSIBLE | med | - | inside .map on objects.get(&target_creature); object present ⇒ calc_chars Some; unwrap_or_else LKI fallback is dead branch |
| 5478 | `OBJECTS_GET` | IMPOSSIBLE | high | - | target verified present on battlefield at 5464 with no intervening zone change |
| 5672 | `ADD_OBJECT` | IMPOSSIBLE | high | - | add_object errs only ZoneNotFound(Battlefield); Err=>continue silently skips Myriad token |
| 5735 | `OBJECTS_GET` | IMPOSSIBLE | high | - | suspended_card verified present in exile at 5725 with no intervening move |
| 5804 | `MOVE_ZONE` | IMPOSSIBLE | high | - | guarded still_in_exile; move to Stack cannot ObjectNotFound and Stack zone exists; Err arm is dead |
| 5831 | `PLAYERS_GET` | IMPOSSIBLE | high | - | players never removed; owner always present |
| 5846 | `LET_UNDERSCORE` | NONSWALLOW | high | - | let _ = is_creature silences unused-var; flag consumed later via was_suspended |
| 5881 | `ZONES_GET` | IMPOSSIBLE | high | - | zones never lose entries; Library(controller) always present; unwrap_or_default masks absence |
| 5901 | `MOVE_ZONE` | IMPOSSIBLE | high | - | exile_card_id from library top_ids with no intervening move; Err arm masks impossible failure |
| 5904 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_exile_id from move_object_to_zone Ok immediately above |
| 5925 | `LET_UNDERSCORE+MOVE_ZONE` | IMPOSSIBLE | med | - | card_id from library-read `remaining`, still in library; reorder move cannot fail; let _ discards |
| 5984 | `LET_UNDERSCORE+MOVE_ZONE` | IMPOSSIBLE | high | - | matching_card found in library at 5974 with no intervening move; let _ discards a Result that is always Ok |
| 5990 | `ZONES_GET` | IMPOSSIBLE | high | - | zones never removed; Library(target_player) present; if-let-else silently does nothing |
| 6003 | `LET_UNDERSCORE+MOVE_ZONE` | IMPOSSIBLE | med | - | card_id from zone.object_ids(), all still in library; reorder move cannot fail; let _ discards |
| 6031 | `ZONES_GET` | IMPOSSIBLE | med | - | zones.get(Library) never None; top()=None is legal empty-library fizzle — assert zone existence only, NOT top_card |
| 6035 | `MOVE_ZONE` | IMPOSSIBLE | high | - | card_id from zone.top() with no intervening move; if-let-Ok silently drops impossible Err |
| 6108 | `MOVE_ZONE` | NONSWALLOW | high | - | error propagated with `?`; guarded present in expected zone |
| 6110 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id from move_object_to_zone Ok above; objects.get None impossible (card_id None is a separate legit and_then) |
| 6111 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id from move Ok above, no intervening move |
| 6349 | `ADD_OBJECT` | NONSWALLOW | high | - | error propagated with `?` |
| 6351 | `OBJECTS_GET` | IMPOSSIBLE | high | - | token_id from add_object Ok immediately above |
| 6574 | `ADD_OBJECT` | NONSWALLOW | high | - | error propagated with `?` |
| 6576 | `OBJECTS_GET` | IMPOSSIBLE | high | - | token_id from add_object Ok immediately above |
| 6815 | `ADD_OBJECT` | NONSWALLOW | high | - | error propagated with `?` |
| 6817 | `OBJECTS_GET` | IMPOSSIBLE | high | - | token_id from add_object Ok immediately above |
| 6908 | `CALC_CHARS` | IMPOSSIBLE | med | - | inside .map on objects.get(&source_object); present ⇒ calc_chars Some; None branch/unwrap_or_default is dead |
| 6935 | `MOVE_ZONE` | IMPOSSIBLE | high | - | source_object verified present, no intervening move; redirect `to` is a valid zone; if-let-Ok drops impossible Err |
| 6965 | `MOVE_ZONE` | IMPOSSIBLE | high | - | source_object present; Graveyard(owner) exists; if-let-Ok drops impossible Err |
| 6980 | `MOVE_ZONE` | IMPOSSIBLE | high | - | source_object present; Graveyard(owner) exists; if-let-Ok drops impossible Err |
| 7029 | `OBJECTS_GET` | FIZZLE | high | CR 608.2b | Mutate target payload; else-branch legally resolves as normal creature spell when target illegal/gone |
| 7032 | `CALC_CHARS` | NONSWALLOW | high | - | Inside if-let-Some(target); calc None impossible and unwrap_or_else supplies a real fallback |
| 7051 | `MOVE_ZONE` | NONSWALLOW | high | - | move_object_to_zone error propagated with `?` |
| 7052 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id just returned by move at 7051, no intervening move; get_mut None would drop field init |
| 7093 | `OBJECTS_GET` | NONSWALLOW | med | - | Merge branch, target present by legality gate; else supplies empty Vector (also a legit first-merge value) |
| 7099 | `OBJECTS_GET` | NONSWALLOW | high | - | and_then yields Option<CardId> carried into MergedComponent.card_id; card_id legitimately optional |
| 7154 | `OBJECTS_GET` | IMPOSSIBLE | med | - | get_mut(target) applies the merge; target present from 7029 gate with only reads intervening |
| 7165 | `OBJECTS_GET` | NONSWALLOW | med | - | .map yields Option<zone> used to locate spell source's zone; None tolerated, objects.remove follows anyway |
| 7167 | `ZONES_GET` | IMPOSSIBLE | high | - | zones.get_mut on the spell's own zone; zones never lose entries, None would skip source removal |
| 7197 | `OBJECTS_GET` | FIZZLE | high | CR 113.7a | TransformTrigger permanent payload; absence = left battlefield, CR 701.27c says do nothing |
| 7211 | `OBJECTS_GET` | IMPOSSIBLE | high | - | get_mut(permanent) after 7197 confirmed present+on-bf with only reads between; None drops the transform |
| 7247 | `MOVE_ZONE` | NONSWALLOW | high | - | move_object_to_zone error propagated with `?` |
| 7248 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_id just returned by move at 7247, no intervening move; None drops field init |
| 7254 | `OBJECTS_GET` | NONSWALLOW | high | - | and_then yields Option<CardId> for ETB; card_id legitimately optional, carried forward |
| 7288 | `OBJECTS_GET` | FIZZLE | high | CR 400.7 | TurnFaceUpTrigger permanent payload; absence/off-bf legally does nothing |
| 7319 | `OBJECTS_GET` | FIZZLE | high | CR 400.7 | DayboundTransformTrigger permanent payload; absence = left battlefield, do nothing |
| 7321 | `CALC_CHARS` | NONSWALLOW | high | - | Inside if-let-Some(permanent); calc None impossible and or_else/unwrap_or_default supply fallback |
| 7330 | `OBJECTS_GET+UNWRAP` | NONSWALLOW | high | - | .unwrap() inside if-let-Some(permanent) at 7319; already assert-like, absence impossible |
| 7346 | `OBJECTS_GET+UNWRAP` | NONSWALLOW | high | - | .unwrap() inside still_needs_transform block; permanent confirmed present, already assert-like |
| 7349 | `OBJECTS_GET` | IMPOSSIBLE | high | - | get_mut(permanent) confirmed present at 7319 with only reads between; None drops the transform |
| 7391 | `OBJECTS_GET` | IMPOSSIBLE | high | - | get_mut(source_object) inside still_on_bf gate (7384); None silently skips class_level set |
| 7505 | `OBJECTS_GET` | FIZZLE | high | CR 400.7 | Delayed-trigger target may have left exile (comment cites CR 400.7); Option gates a legal no-op |
| 7514 | `MOVE_ZONE` | IMPOSSIBLE | med | - | if-let-Ok swallows Err, but target confirmed in Exile at 7507 + Battlefield zone always exists → Err impossible |
| 7516 | `OBJECTS_GET` | IMPOSSIBLE | high | - | new_bf_id just returned by move at 7514; None drops controller/tapped init |
| 7564 | `MOVE_ZONE` | IMPOSSIBLE | med | - | if-let-Ok swallows Err, but target confirmed in Exile at 7560 + Hand(owner) zone always exists → Err impossible |
| 7585 | `MOVE_ZONE` | IMPOSSIBLE | med | - | if-let-Ok swallows Err, but target confirmed in Graveyard at 7581 + Hand(owner) always exists → Err impossible |
| 7615 | `CALC_CHARS` | NONSWALLOW | high | - | Inside .map on present target (filter battlefield); calc yields Option carried as pre_chars with fallback |
| 7654 | `MOVE_ZONE` | IMPOSSIBLE | med | - | if-let-Ok swallows Err, but target confirmed on-bf at 7609 + Graveyard(owner) always exists → Err impossible |
| 7682 | `CALC_CHARS` | NONSWALLOW | high | - | Inside .map on present target; calc Option carried as lki_power with .or fallback |
| 7689 | `MOVE_ZONE` | IMPOSSIBLE | med | - | if-let-Ok swallows Err, but target confirmed on-bf at 7679 + Exile zone always exists → Err impossible |
| 7756 | `ADD_OBJECT` | IMPOSSIBLE | high | - | add_object errors only ZoneNotFound; Battlefield always exists → swallowed Err is a bug |
| 7785 | `ADD_OBJECT` | IMPOSSIBLE | high | - | add_object errors only ZoneNotFound; Battlefield always exists → swallowed Err is a bug |
| 7806 | `LET_UNDERSCORE` | NONSWALLOW | high | - | `let _ = recipient` silences unused var for deferred gift type; nothing swallowed |
| 7858 | `OBJECT_ACC` | NONSWALLOW | high | - | state.object(source_object)? propagates error with `?` |
| 7869 | `MOVE_ZONE` | NONSWALLOW | high | - | move_object_to_zone error propagated with `?` |
