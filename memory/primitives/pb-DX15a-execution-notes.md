# PB-DX15a — execution notes

**Task**: `scutemob-216` · v4 queue rank 3 (`memory/primitives/seed-rerank-2026-08-14.md` §4 row 3)
**Seeds**: `OOS-DP9-8` (CR 608.2e/701.22c/701.23i APNAP) + `OOS-DP9-11` (CR 400.7 same-zone renumber)
**Riders named by the memo**: `OOS-DX24-1`, `OOS-DX24-7`
**Explicitly NOT taken**: `OOS-DP9-16` (parked — unreachable by construction; there is no PB-DX15b)

---

## §0 — Wire prediction, written BEFORE any code changed

> This section was written and committed before a single non-test source line moved.
> Commit order is the evidence: this file's first commit precedes every code commit
> on the branch. The v4 memo's row 3 cell says **none (HIGH)** for both halves.

**Prediction: `PROTOCOL_SCHEMA_FINGERPRINT` UNMOVED and `HASH_SCHEMA_VERSION` UNMOVED,
for both halves.** Derivation, per half, rather than inherited from the memo:

**APNAP half (`OOS-DP9-8`).** The change reorders the *iteration* of an existing
`Vec<PlayerId>` produced by `effects::resolve_player_target_list` and its siblings. It
adds no type, no enum variant and no struct field. `PlayerTarget` is unchanged;
`EffectChoiceQuestion` / `EffectChoiceAnswer` are unchanged; no `Command` or `GameEvent`
gains or loses a shape. `rules::abilities::apnap_order` already exists and is already
called from `rules/engine.rs:1617` and `rules/abilities.rs:8374`, so nothing new becomes
reachable from a closure root. **HASH**: `hash.rs` hashes declared *shapes*; a different
ordering of the same player ids changes the runtime `public_state_hash` **value** on
affected games but not the *schema*, and `HASH_SCHEMA_VERSION` gates the schema.

**Same-zone half (`OOS-DP9-11`).** Replacing a same-zone `move_object_to_zone` /
`move_object_to_bottom_of_zone` with a `Zone::reposition_within`-style permutation
mutates `GameState.zones` (an existing field of an existing type) and *declines* to
mutate `objects` / `timestamp_counter`. `Zone` is unchanged. No new type.

**Stop condition (binding).** If either gate moves, that is a signal to **stop and
re-scope**, not to edit the pin. Both gates are executed after the implement phase and
the measured value is recorded in §7 below, taken from the gate's own output.

**What WILL move, and is budgeted rather than discovered:**
- Golden scripts whose per-step assertions read `ObjectId`s across a same-zone reorder,
  or whose event order is per-player (APNAP reorders the questions/events).
- SR-9b per-step fingerprints, for both reasons: fewer `ObjectId`s minted (the same-zone
  half) and a different event order (the APNAP half).
- Any seeded fixture whose shuffle/coin-flip outcome depends on `timestamp_counter`,
  because the same-zone half stops consuming values from it.

Every moved pin is listed **by name with its CR reason** in §6. A pin repaired by
weakening an assertion is a defect, not a repair.

---

## §1 — Census (both populations are FLOORS — dispatch hygiene 6)

Both populations were re-derived at HEAD by an inverse method, treating the filed lists
as floors.

**Correction (`/review` Issue 3).** The first draft of this sentence named **two** roster
files, one of which — `crates/engine/tests/core/pb_dx15a_apnap_choice_roster.rs` — **does
not exist**. That is the exact failure this batch's own roster file warns about
(`pb_dx15a_same_zone_identity_roster.rs:21`: *"a doc comment asserting a test that does
not exist is a claim like any other"*), committed in the note describing the file that
says it. So, precisely:

- The **same-zone** population is **PRINTED** by
  `crates/engine/tests/core/pb_dx15a_same_zone_identity_roster.rs`
  (`t_total_population_report` and the four family rows R1-R4), so a reader can
  re-derive it rather than trust §1.1.
- The **APNAP** figures in §1.2 are **prose, derived by reading**, not printed by a test.
  The site list is a source-level claim and the deck-legal figures were counted by hand.
  They are now behaviourally *covered* — every one of the five fixed sites has a probe
  (`pb_dx15a_apnap_sites.rs` plus `pb_dx15a_apnap_channel.rs`) — but "covered" is not
  "printed", and the difference is stated here rather than papered over.

### §1.1 — Same-zone renumber (`OOS-DP9-11`): the filed 5 is a FLOOR, short by 12

The v4 memo and the registry row both name **five** deck-legal `Complete` defs
(`birthing_ritual`, `chaos_warp`, `goblin_ringleader`, `growing_rites_of_itlimoc`,
`sylvan_messenger`). All five reproduce at HEAD — none is stale — **and they are one of
FOUR engine mechanisms that reach a same-zone move.** Measured population: **17**
deck-legal `Complete` defs, plus 5 more that would reach the moment they are promoted.

| family | engine site | deck-legal `Complete` members | in the memo? |
|---|---|---|---|
| A — `RevealAndRoute` / `LookAtTopThenPlace` routing to a library | `effects/mod.rs` `:6303` (bottom), `:6305` (**top**), `:6456` | **5** — the memo's list, exactly | yes |
| B — `SearchLibrary` with `destination: ZoneTarget::Library` | `effects/mod.rs:4153` | **8** — `elvish_harbinger`, `enlightened_tutor`, `forerunner_of_the_legion`, `imperial_seal`, `insatiable_avarice`, `mystical_tutor`, `vampiric_tutor`, `worldly_tutor` | **no** |
| C — `KeywordAbility::Hideaway` | `resolution.rs:6350` | **1** — `windbrisk_heights` | **no** |
| D — `KeywordAbility::PartnerWith` | `resolution.rs:6429` | **3** — `brallin_skyshark_rider`, `pir_imaginative_rascal`, `toothy_imaginary_friend` | **no** |

Three corrections to the filed row, each of which matters to someone reading it:

1. **`chaos_warp` reaches the `LibraryPosition::Top` branch (`:6305`), not the bottom
   helper.** The row is filed against `move_object_to_bottom_of_zone`; Chaos Warp's
   revealed non-permanent card stays on **top** and is renumbered by
   `move_object_to_zone`. A sweep scoped to the bottom helper alone would have missed
   the memo's own third-named def.
2. **Family D's blast radius is not "1-3 cards", it is the whole library.**
   `resolution.rs:6429` permutes `zone.object_ids()` for the *entire* target library by
   moving every id to the bottom in turn — so a 99-card Commander library used to mint
   **99** fresh `ObjectId`s and burn **99** `timestamp_counter` values on **every**
   Partner-With ETB, invalidating every pre-existing library id in the game. It runs
   *"whether found or not"* (`:6404-6412`), i.e. unconditionally.
3. **The two censuses do not nest.** An oracle-text axis over all 1,803 defs (the
   printed shapes: "put the rest on the bottom of your library", "look at the top N")
   finds family A and **cannot see** families B/C/D — a tutor's printed text never says
   "put the rest on the bottom", and a keyword confers through
   `AbilityDefinition::Keyword`, not through an `Effect` payload. Conversely the
   structural axis over `Effect` payloads cannot see family C/D at all. This is
   PB-DX26's and PB-DX43's lesson arriving a third time: **a roster derived from one
   declaration construct measures that construct.** Both axes are standing roster rows.

**Why a per-caller sweep was rejected.** `Effect::MoveZone` and `Effect::PutOnLibrary`
resolve their destination from a `ZoneTarget` **at runtime**, so "is this call
same-zone" is not a property of the call site at all — it is a property of the card that
happens to be resolving. The guard therefore lives inside the two `GameState` move
helpers, which makes a renumbering same-zone move *unrepresentable* rather than merely
absent. That is the difference between a sweep and a fix, and it is why the criterion's
"or a sibling" clause was taken.

### §1.2 — APNAP (`OOS-DP9-8`): the seed's own pin was VACUOUS, and its headline family is mis-framed

**(a) The recorded deviation was recorded in the one configuration that cannot express it.**
`test_dp9_choice_inside_for_each_each_player` carried a "Recorded deviation (OOS-DP9-8)"
block ending *"Not fixed here; this test asserts the order the engine actually has."* It
ran on `fixture` — two seats, `.active_player(p(1))` — and asserted `vec![p(1), p(2)]`.
`GameStateBuilder` seeds `turn_order` in `add_player` call order, which is ascending in
every fixture in this repository, so **APNAP starting from the lowest `PlayerId` IS
ascending `PlayerId`**: rotating a list to start at its first element is the identity.
The pin would have stayed green under either rule. That is why the seed survived from
PB-DP9 (`scutemob-157`) to here — the suite reported a pinned deviation while pinning
nothing.

The same vacuity affects `fixture_3p` (also `.active_player(p(1))`) and
`pb_eng1_effect_discard_choice.rs`'s 3-player discard-order test, whose prose at `:417`
says the engine "iterates players ascending". Four stale or unfalsifiable APNAP claims
in the tree, not one.

It is now stated **structurally** rather than left as a discovered fact:
`test_dx15a_active_lowest_id_makes_apnap_and_ascending_indistinguishable` asserts the
coincidence over 2..=6 seats, plus the contrasting non-vacuous case, so the same mistake
cannot recur silently.

**(b) The seed names ONE function; there are seven order-sensitive sites.**

| site | file | status |
|---|---|---|
| `resolve_player_target_list` — `EachPlayer` / `EachOpponent` | `effects/mod.rs:8201` | **the seed's only named site**; fixed |
| `resolve_effect_target_list` — `EffectTarget::EachPlayer` / `EachOpponent` | `effects/mod.rs:~7999` | independently reachable, same `OrdMap`, same defect; fixed |
| `Effect::ForEach` | `effects/mod.rs:4520` | delegates to the above; fixed transitively |
| `Effect::LivingDeath` | `effects/mod.rs:7511` | fixed — see (c) |
| `Effect::ReturnAllFromGraveyardToBattlefield` | `effects/mod.rs:7378` | fixed; the walk decides the order permanents enter and therefore the order ETB triggers are queued |
| `Effect::Manifest` / `Effect::Cloak`, `EachOpponent` arm | `effects/mod.rs:5116`, `:5182` | **NOT fixed — see §4.3**; these collapse "each opponent" to a single `.keys().find(..)` pick, a different and larger defect |
| `resolve_cda_player_target` | `layers.rs:3093` | **deliberately not fixed** — CDA context, order-insensitive (its consumers count and sum); documented at the function |

Fixing only the function the seed named would have left `resolve_player_target_list` and
`resolve_effect_target_list` **disagreeing about the order of the same set of players**.

**(c) Two comments asserted APNAP that the code did not implement.** `Effect::LivingDeath`
read `state.players.keys().copied().collect()` then `.sort()` under a comment saying
*"Determine APNAP player order (active player first, then in turn order)"*; the
`Effect::WheelHand` arm said *"APNAP order comes from `resolve_player_target_list`'s
`PlayerTarget::EachPlayer` iteration"*. The `OOS-DX28-6` note-vs-code shape, twice,
**inside this batch's own subject matter**. Both are now true, and both are called out
in-source rather than quietly corrected.

**(d) The memo's headline family is mis-framed, and the correction is the load-bearing
part.** Row 3 says the batch "repairs the Fleshbag/Grave Pact family (10 defs)", which
reads as ten defs regaining a per-player choice. Measured:
`effects::sacrifice_permanents_for_player` (`effects/mod.rs:9479-9492`) computes
`eligible_sacrifice_targets`, **sorts, and takes the first `n`** — it asks nothing. So
`Effect::SacrificePermanents { player: EachPlayer }` has no per-player *question* at all,
and what this batch fixes for that family is the **order the sacrifices happen** (event
order, and hence trigger-queueing order), not a choice. The agency gap — nobody chooses
which creature — is a separate, pre-existing defect that PB-DX15a does **not** close, and
the probe covering that family says so in its own doc rather than implying otherwise.

Only **2** deck-legal `Complete` defs (`burglar_rat`, `geier_reach_sanitarium`) exercise
the seed's literal per-player-question claim; 3 more (`echo_of_eons`,
`whirlpool_warrior`, `winds_of_change`) reach it through the shared RNG counter.

**(e) Neither CR violation was pinned anywhere in the 4,797-test suite.** A full
`--workspace --no-fail-fast` run with **both** engine halves in place is **4,797 / 0 / 5**
— zero failures, no golden script moved, no SR-9b fingerprint moved, no seeded constant
re-observed. §0 budgeted that movement and **none came due**; that is stated here as a
paid-and-unclaimed budget rather than quietly dropped. The consequence for this batch is
strict: there is **no inherited red-before evidence anywhere**, so every probe must be
proven red by an executed revert (§5).

## §2 — APNAP half

## §3 — Same-zone half

## §4 — Rider and park dispositions

### §4.1 — `OOS-DP9-16`: **NOT TAKEN**, parked, as the brief directs

The v4 memo parks it as *unreachable by construction — both delayed-trigger producers
mint fresh `ObjectId`s*, and states there is no PB-DX15b. It is **not** taken here, and
this sentence is the record of that decision rather than an omission. Nothing in this
batch touches `turn_actions.rs`'s end-step delayed-trigger sweep.

One coupling worth writing down for whoever does take it: this batch removes a source of
`ObjectId` minting (same-zone moves). `OOS-DP9-16`'s "unreachable by construction"
argument rests on delayed-trigger producers minting *fresh* ids so two delayed triggers
cannot share a `target_object` key. **This batch does not weaken that argument** — the
producers it names are zone CHANGES, which still mint — but the argument is now one
mechanism narrower than it was, and a future batch that removes another minting site
should re-check it rather than inherit it.

### §4.4 — `OOS-DX24-1` (`doubler_applies_to_trigger` source-blind): **DEFERRED**, and the row's prescribed fix is CR-WRONG as written

**Disposition: DEFERRED. Reason: the fix the row prescribes, implemented verbatim, ships
a regression on the most common real interaction the arm it touches exists to serve.
Proven by execution, not argued.**

The row prescribes *"one source-zone conjunct at the top of `doubler_applies_to_trigger`
(`abilities.rs:10020`), before the `match`, covering all four arms at once"*, on the
CR 110.1 reasoning that every printed doubler says "a triggered ability of a **permanent**
you control" and a card in a graveyard is not a permanent. The reasoning is right; the
conjunct does not implement it.

**Why zone alone cannot decide this.** A CR 603.6c / 603.10a look-back "when this dies"
trigger is constructed at `abilities.rs:4740` as
`PendingTrigger::blank(*new_grave_id, *death_controller, kind)` — its `source` is the
dying creature's **graveyard** object, because `move_object_to_zone` has already removed
the battlefield object from `state.objects` by trigger-check time. So at doubling time:

| case | correct verdict | source's zone |
|---|---|---|
| Teysa Karlov doubling a creature's own "when this dies" trigger | **double it** — CR 603.10a: it *was* a permanent's ability | Graveyard |
| Nether Traitor's `trigger_zone: Graveyard` ability (the seed's own subject) | **do not double it** — it was never a permanent's ability | Graveyard |

**Both present a graveyard source. The conjunct cannot separate them, and it silently
picks the wrong one for the common case.**

**The experiment (revert row R1).** `trigger_doubling.rs` carried **nine** tests before
this batch and **not one** exercised the `CreatureDeath` arm — every one is an ETB arm.
So this batch first wrote
`test_dx15a_creature_death_doubler_doubles_a_look_back_dies_trigger`, confirmed it GREEN
at HEAD (the engine gets this right today), then applied the row's prescribed conjunct
verbatim and ran the file:

```
test_dx15a_creature_death_doubler_doubles_a_look_back_dies_trigger ... FAILED
  left: 1
 right: 2
9 passed; 1 failed
```

**The nine pre-existing tests all stayed green.** That is the durable half: the arm had
zero behavioural coverage, so the prescribed fix would have shipped with the workspace
green and nobody the wiser. The conjunct was then removed and the file restored to
10 passed / 0 failed.

**What a correct fix needs, and why it is out of scope here.** The discriminator is not
the source's zone but *why* it is there — whether it arrived in the graveyard as part of
this very event. That information exists in exactly two places, and neither is available
to this batch:
1. `check_triggers`' `arrived_in_graveyard_this_batch` set — which is not in scope at
   `compute_trigger_doubling`'s call site (the doubling happens at flush time, long
   after);
2. a construction-time marker on `PendingTrigger` — which is a **hashed, serialized**
   type, so that is a `HASH`/`PROTOCOL` bump. This PB predicted NONE in writing before
   any code (§0), measured NONE, and the project's standing rule is one wire bump per PB.

Deferring costs nothing live: the row's own corrected measurement is **zero deck-legal
pairings in either direction**.

**What ships instead of the fix**: the probe. It is the first behavioural coverage the
`CreatureDeath` arm has ever had, and it is written wrong-way-round on purpose — whoever
takes `OOS-DX24-1` must keep it green, which rules out the prescription the row currently
carries. The row is corrected in the registry to say so.

### §4.2 — `OOS-DX24-7` (CR 603.10a look-back set coarser than one batch): **TAKEN** — and the row's fix sketch was INVERTED

**Disposition: TAKEN.** Implementation: `rules::abilities::EventBatchTiming` +
`check_triggers_with_timing`. Probe: `crates/engine/tests/rules/pb_dx15a_lookback_batch_timing.rs`
(t1-t4). Revert rows R3/R4 in §5.

The row's fix sketch is *"rebuild the set per event **prefix** rather than per whole
slice, so each event looks back only at deaths strictly earlier in `events`' order."*
**Two things are wrong with it, and both were settled by executing the sketch rather than
by reasoning about it.**

**(1) Applied to every caller, the prefix makes `sba.rs` wrong — the caller the guard was
written for.** Within one CR 704.3 fixpoint pass the deaths are genuinely simultaneous,
so CR 603.10a's "immediately prior" means prior to **all** of them. That is not an
inference: it is the Gatherer ruling `check_triggers` already quotes in-source — *"If
Nether Traitor and another creature are put into your graveyard **at the same time**,
Nether Traitor's ability won't trigger."* A prefix set there makes a simultaneous batch's
answer depend on the slice's incidental ordering, which is a property the batch does not
have. `t2` asserts both orderings and pins it.

So timing became a **caller declaration** rather than a property of whatever slice a
caller happened to hand in:

| call site | timing | why |
|---|---|---|
| `sba.rs:97` | `Simultaneous` | PB-DX24 measured this one EXACT: one CR 704.3 fixpoint pass |
| `resolution.rs:8248` | `Sequential` | PB-DX24 measured this one COARSE: a whole resolution's sub-effects, which run in sequence |
| `combat.rs` ×2, `engine.rs` ×2 | `Simultaneous` | **byte-identical to their previous behaviour.** PB-DX24 recorded these four as NOT audited and this batch did not audit them either. The parameter is what makes that status visible *at the call site*, and each carries a comment saying so — the alternative was leaving four callers silently inheriting a default |

**(2) The prefix is what to SUBTRACT, not what to pass.** The set is a *suppression* set:
a source in it did **not** yet have a functioning graveyard ability immediately prior to
the event. A source that arrived at an **earlier** event was already there, so it must be
**removed**. Passing the prefix itself inverts the guard — and on the row's **own worked
example** (a resolution that sequentially puts a `trigger_zone: Graveyard` source into a
graveyard, then kills another creature) the prefix at the second event is `{source}`,
which suppresses. **The row's sketch reproduces the very defect the row describes.**
Revert row R3 applies it verbatim: `t1` **and** `t3` go red.

The shipped set is `whole_batch − strictly_earlier_arrivals`. The subtraction is also
what keeps the *other* order correct: `check_triggers` runs **after** every event in the
slice has been applied, so a source arriving later in the slice is already sitting in the
graveyard when `collect_graveyard_carddef_triggers` enumerates `state.objects`. Keeping
later-and-current arrivals in the set is what stops it firing off a death that happened
before it got there — `t3`, and revert row R4 (subtract everything) reddens exactly that.

**A gate caught this batch's own work and was right.** PB-DX7's
`unordered_iteration_ratchet` fired on the first draft, which added three `HashSet`s and
pushed `rules/abilities.rs` from 11 to 15. They are `contains`-only, i.e. legitimately
the ratchet's category (a) — "raise the ceiling and say which". Converting to `BTreeSet`
was taken instead: it costs nothing at this size, moves the ceiling **down** (11 → **6**,
re-pinned with the reason in the entry) rather than asking for a raise, and removes the
question entirely from a function PB-DP9 re-executes wholesale after every suspended
`EffectChoiceQuestion` (`OOS-DP9-10`).

**Both riders' prescriptions were wrong as written, in different ways, and neither
would have been caught by reading.** `OOS-DX24-1`'s ships a regression under a green
workspace (§4.4); `OOS-DX24-7`'s reproduces its own defect. The common cause is worth
naming: **a fix sketch written from the symptom describes the symptom's neighbourhood,
not the mechanism** — and this repository's rule that a row is a claim like any other
applies to the *fix* half of a row, not only to its measurement half.

### §4.3 — `Effect::Manifest` / `Effect::Cloak`'s `EachOpponent` arm: NOT taken, filed

Found while enumerating the APNAP sites (§1.2b), outside both riders' scope.
`effects/mod.rs:5116` and `:5182` handle `PlayerTarget::EachOpponent` by taking
`state.players.keys().find(|&&pid| pid != ctx.controller)` — i.e. they collapse "each
opponent" to a **single** opponent, the lowest `PlayerId`. Two defects in one expression:
the cardinality is wrong (a bigger defect than the ordering), and the single pick is
ascending rather than APNAP.

Not fixed, and not fixed for a stated reason rather than an unstated one: **corpus reach
is zero**. The three `Effect::Manifest`/`Effect::Cloak` users (`cryptic_coat`,
`reality_shift`, `write_into_being`) are all `PlayerTarget::Controller`; **no def in the
corpus reaches the `EachOpponent` arm at all.** Changing which single opponent is picked
would be motion on a path nothing takes, on top of a cardinality bug this batch is not
scoped to fix. Filed as a new seed instead (§6), so the *cardinality* half — the one that
matters — is what the next batch is pointed at, rather than the ordering half being
quietly patched and the row closed.


## §5 — Revert matrix

**Every gate and probe in this batch was proven RED by an executed revert.** That is not
the usual belt-and-braces here — it is forced. §1.2e records that a full `--workspace`
run with both engine halves in place is **4,797 / 0 / 5**: neither CR violation was
pinned anywhere in the suite, so **no probe in this batch inherits red-before evidence**
and each has to earn its own.

### A — the APNAP half (`OOS-DP9-8`)

Revert: `crate::rules::abilities::apnap_order_all_players(state)` →
`state.players.keys().copied().collect::<Vec<_>>()`.

| row | reverted arm | observed |
|---|---|---|
| **A1** | `PlayerTarget::EachPlayer` | **RED** — `test_dx15a_each_player_search_asks_in_apnap_order`: `left: [1, 2, 3]`, `right: [2, 3, 1]` |
| **R1** | `EachPlayer` only | **RED** on `c1` (human `LocalGame`), `c2` (bot path), `c4` (Fleshbag resolution order) |
| **R2** | `EachOpponent` only | **RED** on `c3` (`burglar_rat`, asked order AND `CardDiscarded` order) and on the play-server HTTP probe |
| **R3** | both | **RED** on `c1`, `c2`, `c3`, `c4`, HTTP |

Observed reds at R3: `[1,2,3]` vs `[2,3,1]` for `c1`/`c2`/`c4`; `[1,3]` vs `[3,1]` for
`c3`; `[]` vs `[3]` and `["Human-1","Bot-3"]` vs `["Bot-3","Human-1"]` for the two HTTP
assertions.

**Two rows are green by design and both are disclosed rather than left to look like a
hole**: `c5` is a **CONTROL** (it reads `apnap_order_all_players` and the fixture
directly, not the `effects` wiring, so it must stay green under every revert), and the
HTTP probe is `EachOpponent`-only, so `R1` leaves it green — that is the arm it does not
exercise. The HTTP probe's second assertion is masked by its first firing sooner, so it
was proven **separately**, with the first neutralised, rather than assumed.

### B — the same-zone half (`OOS-DP9-11`)

| row | edit | reds |
|---|---|---|
| **V1** | disable the `from == to` guard in `move_object_to_zone` | 5 |
| **V2** | same, in `move_object_to_bottom_of_zone` | 9 |
| **V3** | swap the `ZoneEnd::Top` / `Bottom` arms | 8 (incl. the `pb_os8` edit) |
| **V4** | `reposition_within_own_zone` returns the id but repositions nothing | 8 (incl. the `pb_os8` edit) |
| **V5** | delete the Hideaway LCG's new `timestamp_counter += 1` | 1 |
| **V6** | add a 6th `next_object_id()` in `move_object_to_bottom_of_zone` | 1 (`r5`) |
| **V7** | rename `reposition_within_own_zone`, behaviour unchanged | 1 (`r5`) |
| **V8-V11** | one card def leaves each family (sylvan_messenger, worldly_tutor, windbrisk_heights, pir_imaginative_rascal) | 3 / 3 / 2 / 2 |

**V1 and V2 could not delete the guard outright**, and that is a real structural property
of the fix rather than a testing inconvenience: deleting it leaves `ZoneEnd::Top` and
`ZoneEnd::Bottom` unconstructed, and the crate's `deny(warnings)` turns that into a
**compile** error, not a red test. They are disabled at runtime instead.

### C — the riders

| row | edit | observed |
|---|---|---|
| **R1 (DX24-1)** | apply `OOS-DX24-1`'s prescribed source-zone conjunct **verbatim** | **RED** — `left: 1, right: 2`, **with all nine pre-existing `trigger_doubling.rs` tests still green** |
| **R3 (DX24-7)** | apply `OOS-DX24-7`'s prescribed "pass the prefix" sketch **verbatim** | **RED on t1 AND t3** |
| **R4 (DX24-7)** | subtract everything (empty sequential set) | **RED on t3** |

### D — honestly UNDISCRIMINATED

**One row**, and it is disclosed **in the test's own doc comment**, not only here:
`t_worldly_tutor_with_nothing_to_find_consumes_only_the_spell_move`. It is a control that
by construction contains no same-zone move, which is precisely what lets `2 − 1 = 1`
prove that the sibling's second counter draw is the shuffle seed rather than the
placement. Strengthening it would destroy the property it exists to establish.

### E — gates that fired on this batch's own work, and were right

| gate | what it caught | disposition |
|---|---|---|
| SR-25 `bare_lookup_ratchet` | the new `reposition_within_own_zone` used a bare `.zones.get_mut(..)` | switched to `expect_zone_mut` — a `None` there is an engine bug |
| PB-DX7 `unordered_iteration_ratchet` | the rider's first draft took `rules/abilities.rs` from 11 to 15 `HashSet`s | converted to `BTreeSet`; ceiling **lowered** 11 → 6 rather than raised |
| the batch's own scry probe | its non-vacuity floor caught the first draft predicting the bottomed card from the fixture's push order, with the convention backwards | captured from the announcement instead |
| the batch's own `r5` roster row | reddened on its first run: `move_object_to_zone` mints **three** ids, not one (two component re-mints for merged mutate/meld components) | counts measured per function with the reason |

**And one probe passed VACUOUSLY before it passed honestly.** The first Worldly Tutor
probe drove `Effect::SearchLibrary` through a bare `execute_effect` and measured a delta
of 0 against an untouched library — because PB-DP9 **suspends and rolls the whole
resolution back** until the choice is answered, so the effect had never run. Rewritten
onto the real stack + `PassPriority` + answer path; disclosed in the test's doc. This is
the `OOS-DP9-*` family's own machinery making a probe of it silently meaningless, which
is worth carrying to the next batch that writes one.

## §6 — Moved pins, by name, with the CR reason

**NONE. The budget written in §0 was paid and nothing came due, and that is stated here
rather than quietly dropped.**

§0 budgeted movement in three places, each for a stated mechanism: golden scripts whose
per-step assertions read `ObjectId`s across a same-zone reorder or whose event order is
per-player; SR-9b per-step fingerprints (fewer `ObjectId`s minted, different event
order); and seeded fixtures whose shuffle or coin-flip outcome depends on
`timestamp_counter`. A full `--workspace --no-fail-fast` run with **both** engine halves
in place produced **zero** failures in any of those categories.

**Why the prediction over-shot, measured rather than guessed:**

- *Golden scripts and deterministic fixtures.* Every fixture in this repository calls
  `add_player` in ascending order and `.active_player(<lowest id>)`, so APNAP and
  ascending `PlayerId` are the same list in all of them (§1.2a). The reorder is
  behaviourally invisible to them **for the same reason the seed's own pin was vacuous**
  — one cause, two consequences.
- *The same-zone half.* It had **no behavioural coverage at all** (§1.1): no test in the
  tree referenced `Zone::reposition_within`, none counted or rostered the move helpers,
  and no Hideaway or Partner-With test asserted an `ObjectId` across a reorder. Nothing
  could move because nothing was looking.
- *Seeded fuzz fixtures.* `pb_dx32_fuzz_output`'s seeds and the play-server's seeded
  constants (`UI3_SPLIT_COMBAT_SEED` and siblings) survived unmoved. The `timestamp_counter`
  trajectory only changes on a game that actually reaches a same-zone move, and the
  4-player/25-turn random-deck fuzz seeds did not reach one.

**Two pins DID move, and neither is in this category — both are pins ON the defects,
handled by inversion rather than repair:**

| pin | what it asserted | disposition | CR reason |
|---|---|---|---|
| `object_identity::test_400_7_same_zone_move_produces_new_id` | `assert_ne!(old_id, new_id)` on a same-zone move | **INVERTED**, renamed `..._keeps_the_same_id`, subject stated in its own doc | CR 400.7's antecedent is *"moves from one zone to another"*. Its stated rationale — *"the zone-change event creates a new object regardless of the source and destination zones being the same"* — inverts the rule it cites |
| `pb_dp9_effect_choice::test_dp9_choice_inside_for_each_each_player` | `asked == vec![p(1), p(2)]`, documented as a recorded APNAP deviation | **INVERTED**, renamed `test_dx15a_each_player_search_asks_in_apnap_order`, rebuilt on a discriminating 3-seat fixture | CR 608.2e / 101.4 / 701.23i. The old assertion was vacuous on its own axis (§1.2a) |

**No assertion was weakened anywhere.** Both inversions are strictly stronger than what
they replaced: the first gained a `timestamp_counter` clause it never had, the second
gained a third seat, a full-order assertion, and an answers-applied-to-the-right-player
clause that fails differently from the order clause.

Two ratchets and one gate fired on this batch's own work, all three correctly, and each
is recorded where it fired rather than only here: SR-25's `bare_lookup_ratchet` (§3),
PB-DX7's `unordered_iteration_ratchet` (§4.2), and the SR-5 keyword registry (no hit this
batch).

## §7 — Gates, measured

| gate | predicted (§0, before any code) | **measured** | how |
|---|---|---|---|
| `PROTOCOL_SCHEMA_FINGERPRINT` | **NONE** | **UNMOVED at 38** | `core protocol_schema` executed: 17 passed / 0 failed |
| `HASH_SCHEMA_VERSION` | **NONE** | **UNMOVED at 77** | `core hash_schema` executed: 36 passed / 0 failed |
| `history_is_append_only` | green | **green** (2/2) | executed |
| `frozen_prefix_is_pinned` | green | **green** (2/2) | executed |

**Prediction and measurement agree on both halves, so the §0 stop-condition never fired
and no pin was edited.** Both gates were executed, not reasoned about; the numbers above
are read off the passing gates rather than transcribed from the previous batch's close
(PB-DX44 closed at PROTOCOL 38 / HASH 77, and this batch reproduces that).

Derivation, restated against what actually shipped: the APNAP half reorders an existing
`Vec<PlayerId>` and adds no type, variant or field; the same-zone half declines to call
`next_object_id()` and mutates only `GameState.zones`, an existing field of an existing
type. The rider's `EventBatchTiming` is a **function parameter** on an engine-internal
function — it is not reachable from `Command`, `GameEvent`, `Effect` or `Characteristics`,
so it is outside the PROTOCOL closure, and it is not a `GameState` field, so it is
outside the HASH schema. That last one was the only addition that could plausibly have
moved a gate, and it was gate-checked rather than assumed.

## §8 — Standard gates, measured at close

**Tests: 4,829 / 0 / 5** full-workspace on branch `scutemob-216`, `--workspace
--no-fail-fast` to a file, **54** result-producing targets (53 → 54: one new simulator
test binary). **+32** over the **4,797** baseline, which was measured on this branch
**before any edit** and reproduced PB-DX44's close pin exactly (4,797 / 0 / 5, 53
targets).

**Delta itemised by test NAME**, by set-diffing the two run logs: **36 additions, 4
names leaving the passing set, 0 removals.** The four are disclosed individually rather
than netted out, because two of them are not what "removed" would suggest:

| name that left | what it is |
|---|---|
| `object_identity::test_400_7_same_zone_move_produces_new_id` | **INVERSION** → `..._keeps_the_same_id` (§6) |
| `pb_dp9_effect_choice::test_dp9_choice_inside_for_each_each_player` | **INVERSION** → `test_dx15a_each_player_search_asks_in_apnap_order` (§6) |
| `crates/engine/src/state/mod.rs - state::GameState (line 81)` | **a doctest, not a test** — its name IS its line number, and the `ZoneEnd` enum added above it shifted it to `(line 91)`. Same doctest, unchanged |
| `crates/engine/src/state/mod.rs - state::GameState (line 90) - compile fail` | the same, shifted to `(line 100)` |

Both doctest shifts are exactly **+10**, which is the height of the `ZoneEnd` declaration
and its doc comment. So the honest reading is **34 genuine additions, 2 inversions, 2
doctest line-number shifts, 0 removals** — and `+32` is the arithmetic of those, not a
figure that hides an edit.

| gate | result |
|---|---|
| `cargo clippy --workspace --all-targets -- -D warnings` | **clean** |
| `cargo fmt --check` | **clean** (0 diffs) |
| `tools/check-defs-fmt.sh` (SR-35) | **clean**, 1,803 defs |
| coverage, regenerated by `tools/authoring-report.py` | **1,136 / 1,803 = 63.0%**, clean 1,136 / todo 520 / empty 147 — **byte-identical counts to PB-DX44's close, 0 flips**, self-dating churn reverted. **1 card-def edit, comment-only** — see below |

**One `cargo fmt --check` failure was shipped and then fixed, and the sequence is worth
recording.** Commit `0b484cb3` landed `effects/mod.rs` with four unformatted filter
closures: `cargo fmt` had been run mid-batch, but the file was later restored from git by
a parallel worker honouring a file-ownership rule, which put the unformatted version
back. It was caught by a *reviewer*, not by me, and it would have failed the workspace
gate at collect. The lesson is narrow and practical: **`cargo fmt --check` must be run
against the FINAL tree, not against the tree as it stood when the code was written.**

**Correction to this section's own first draft.** It said **0 card-def edits**, which was true
when written and false by the time the batch closed: the `/review` fix cycle edited
`crates/card-defs/src/defs/nether_traitor.rs`, whose in-source note cited
`check_triggers`'s look-back set as *the* enforcement of its Gatherer ruling — an enforcement the
`OOS-DX24-7` rider had made conditional on a per-caller timing. The edit is **comment-only**, so
coverage is unmoved and `check-defs-fmt.sh` stays clean (1,803 defs, re-run after the edit), but
"0 card-def edits" is the kind of figure this project treats as a claim, and a claim that goes
stale mid-batch is corrected rather than left standing.
