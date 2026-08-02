# SIM-2 — mana intelligence (`scutemob-176`, 2026-08-02)

Evidence record for the batch that closed playtest-triage **F3**, **F4** and **F5**
(`memory/playtest-triage-2026-08-02.md`), plus **OOS-CARDS2-9** (never filed, only named in
source), the layer half of **OOS-M11-2**, and the bot half of **OOS-M11-8**.

Scope as dispatched: `crates/simulator` only. Actual diff: `crates/simulator` (**4** source
files — `mana_solver`, `local_game`, `legal_actions`, `heuristic_bot` — plus 4 test files) +
`tools/play-server/src/main.rs` (one seed pin) + docs/memory + **one data line in
`crates/engine/src/state/keyword_registry.rs`**. That last one is neither scope
creep nor optional: SR-5's gate (`core::keyword_registry::registry_sites_match_the_source_tree`)
greps the source tree for `KeywordAbility` branches and asserts set equality against the
declared sites, so the solver's new CR 302.6 summoning-sickness check must be declared or the
gate fails. **No engine behaviour changes** — `git diff main -- crates/engine/src/rules
crates/engine/src/effects crates/card-types crates/card-defs` is empty, and PROTOCOL 33 /
HASH 70 are gate-executed unmoved.

---

## 1. What was wrong, in the order it was found

### F4 — the solver counted SOURCES, not MANA

`mana_solver.rs` expanded each ability's `produces` per unit of mana and then never read the
expansion: all three payment phases decremented the remaining pip count by **one per source
tapped**. Sol Ring (`{C}{C}`) was one mana. Both directions were live, and a human saw both
in the browser client:

* **over-tap** — Sol Ring + two Forests for `{2}{G}`: four mana made, three spent, one
  stranded and destroyed at the step boundary (CR 500.4);
* **under-offer** — a `{2}` spell with only a Sol Ring untapped solved to `None`, so
  `legal_actions::can_afford` never offered the cast at all.

The fix is not "multiply by the count": a tapped source now credits its whole production to a
running `Floating` tally, and each pip is paid from that tally before any further source is
tapped, so a multi-mana source's surplus is spendable on the rest of the same payment.

### F3 — auto-tap was all-or-nothing

`LocalGame::auto_tap_commands_for` checked whether the pool covered the **whole** cost; if it
did, it tapped nothing, and if it did not, it handed `solve_mana_payment` the **entire**
printed cost with the pool never subtracted. Two floating + a `{3}` cast tapped three more
sources. `solve_mana_payment_with_pool` now subtracts the pool in `ManaPool::can_spend`'s own
order (colored pips from matching colored mana, `{C}` from colorless only, then the whole
remainder against generic) and solves the residual; the early return is the residual-is-zero
case of the general rule rather than a special case beside it.

`can_afford` had the same shape one layer up — a pool-total shortcut **or** a whole-cost
solve, with nothing covering the middle — so a player with `{G}` floating and one Forest up
was told a `{1}{G}` spell was uncastable. It now asks the residual solver one question, which
is the same function the auto-tapper uses to build the plan.

### F5 — the bot tapped out every empty upkeep

`TapForMana` scored 5, `PassPriority` 1, and nothing conditioned the 5 on having anything to
spend it on. Now 0. **The gate-vs-demote choice is not arbitrary**: every action that can
consume mana already outscores 5 (`PlayLand` 100, `CastSpell` 50+, `ActivateAbility` 40), so
a tap was only ever *chosen* when it was the sole alternative to passing — which is the
empty-upkeep tap-out and nothing else. Scored 0 rather than removed, so it stays choosable
when it is all there is (`t9`).

### The layer half of OOS-M11-2 — recorded as theoretical, actually live-wrong

Changing which source the solver reaches for made
`test_s8_scripted_human_playthrough_is_clean_on_five_seeds` fail on seed 42 with

```
engine rejected a just-offered action (CastSpell):
object ObjectId(487) has no mana ability at index 0
```

`layers.rs` clears `mana_abilities` outright for a **face-down** permanent (CR 707.2), and
the solver read `obj.characteristics.mana_abilities` raw. The doc block in `mana_solver.rs`
had described this gap with a *granted*-ability example (Cryptolith Rite) and no urgency; the
real instance is an ordinary morph/manifest. `gather_sources` now calls
`calculate_characteristics`, the same function `StubProvider`'s offer loop and
`handle_tap_for_mana` both use.

**Measured, not assumed**: `mtg-fuzzer --games 60 --threads 1 --max-turns 40` reports 6.8 s
before and after on seeds 1 and 7, two runs each. That is why the gather is *not* hoisted out
of the solve and handed to `can_afford` as a pre-computed list — a real complication for an
unmeasurable saving.

### OOS-CARDS2-9 — filed nowhere, and half-described

CARDS-2 named it in three `play-server` comments and never put it in the audit inventory. Its
statement of the fix ("one place: make the solver ask whether the ability is activatable")
was right about the affordability half and silent about the **offer** half: `legal_actions`'s
`TapForMana` loop checked `life_cost` and nothing else, so an unmet `activation_condition`
and a summoning-sick creature were offered and refused — and the play-server driver carried
both refusal strings in `KNOWN_FALSE_OFFERS` to drive past them. One predicate,
`tap_ability_is_activatable`, is now called by both.

### Two more, found by this batch's own `/review` — and the second is the sharper one

**Stax restrictions were mirrored nowhere on the tap path.** `rules/mana.rs`'s step 1b
refuses a `TapForMana` under `ArtifactAbilitiesCantBeActivated` (Collector Ouphe, Stony
Silence) and `OpponentsCantCastOrActivateDuringYourTurn` (Grand Abolisher) — CR 605.3 makes
activating a mana ability follow the rules for activating any other activated ability.
Neither the solver nor `StubProvider`'s offer loop read `state.restrictions()`, so with an
opponent's Collector Ouphe out, `can_afford` counted a Sol Ring, the cast was offered, and the
atomic tap-and-cast sequence was refused: the exact 422 this batch exists to remove, one
restriction class away from where it was being fixed. Closed by reusing the provider's own
`is_ability_restricted_by_stax` (SIM-1's lesson: two arithmetics agree only when they are one
function), pinned two-sided by `t22`.

**What makes it worth writing down is not the miss but the claim.** Four separate comments
written by this batch asserted that `plannable_tap_ability` mirrored *every* rejection
`handle_tap_for_mana` makes — "SIM-2 added all of them", "the single gate", "the returned
commands are always ones `handle_tap_for_mana` accepts". An enumeration is only as complete
as the category it names (`OOS-SIM1-3`, one batch earlier, about `GameRestriction` variants;
here the same shape about the *set of rejections in a function*). Those comments now state a
bound instead of an absolute, and name what is still unmirrored.

**A "safe under-count" that over-credits at zero.** SR-36 scaled abilities carry a
`1`-per-colour marker instead of a count, and both the roster gate's doc and `OOS-SIM2-4`
originally said the marker "can only under-offer; it never over-credits". False at zero:
`rules/mana.rs` computes `resolve_amount(..).max(0) as u32` and adds it with **no error**, so
`growing_rites_of_itlimoc` (`Complete`, deck-legal) taps for nothing with no creatures out
while the marker promises one mana — an offered cast the engine then refuses. Scaled abilities
are now excluded from planning outright (`resolve_amount` is `pub(crate)` to the engine, so
the solver cannot ask for the real number); `t23` pins it. The lesson is narrow and useful:
**"conservative" is a claim about a direction, and a direction has to be checked at its
endpoints.**

The *second* pass of the same `/review` then caught the same species of overclaim inside the
fix for the first: the exclusion's comment said it cost "nothing that was working", and that
is false — a Cradle with **one** creature out was offered and the cast *succeeded*, and that
case is now withheld. It is also over-broad for **three** of the roster's nine,
which count a population containing themselves and so cannot reach zero on the battlefield
(`elvish_archdruid`, `priest_of_titania`, `circle_of_dreams_druid`). Carving those out by
name would be a shadow implementation of the count — the trap avoided one paragraph earlier on
the stax fix — so the blunt exclusion stands and the claim was corrected instead. **Twice in
one batch, the thing needing repair was a sentence asserting the code was safe.**

And then a **third** time, in the correction itself: that sentence originally said *four*,
counting `marwyn_the_nurturer` among the safe ones. Marwyn is a **1/1** reading
`EffectAmount::PowerOf(EffectTarget::Source)` — not a population — so one `-1/-1` counter or
any layer-7b P/T setter takes it to zero and it fails exactly as Itlimoc does. It is evidence
*for* the exclusion. The reviewer's own second-pass report had listed it with the hedge "≥1
unless debuffed", and the durable observation is theirs: **a hedge inside a list gets read as
a member of the list.** Re-derived rather than decremented, per this file's own rule about not
adjusting a constant by hand.

One more roster subtlety, since a reader will go looking: the motivating card's scaled ability
is on its **back** face, so `r5`'s nine rows do not contain `growing_rites_of_itlimoc` —
`enrich_spec_from_def` builds front-face ability vectors. The runtime exclusion still covers
it, because `apply_face_change` rebuilds those vectors at the transform.

### The bot half of OOS-M11-8 — closed by there being one function

S8 recorded `OOS-M11-8` (announced `{X}` not paid for) as CLOSED on a fix that lived only in
`auto_tap_commands_for`, while `advance()` kept its own `solve_mana_payment` call on the
taxed printed cost. A bot announcing X > 0 therefore tapped for the base cost and had the
cast refused — latent rather than live, because both shipped bots build their command from
`ActionParams::default()` and announce `x_value: 0`. `advance()` now calls
`auto_tap_commands_for`; `t21` drives a purpose-built `XBot` that announces X = 2 and fails
when `advance()` is reverted to its old body.

---

## 2. Populations (enumerated from `all_cards()`, SR-36 — never grepped)

Pinned by `crates/simulator/tests/sim2_mana_source_roster.rs`, over 1,803 defs. Counted as
**(def × `{T}` ability) rows**, not defs — the first write-up said "322 defs", which is the
wrong unit for every row below it:

| row | population | count |
|---|---|---|
| — | `{T}` mana-ability rows in the corpus | 322 |
| R1 | `{T}` abilities producing **>1** mana (fixed colour, unscaled) | 36 |
| R2 | `{T}` abilities with their own **mana** component | 20 |
| R3 | with a **life** component / with an **activation condition** | 8 / 13 |
| R4 | with a **counter** component | **0** (pinned empty) |
| R5 | SR-36 **scaled** abilities (marker of 1, not a count) | 9 |

R4 is pinned at zero with a non-vacuity floor, because an unexercised filter rots silently:
the arm itself is covered by a synthetic fixture (`t14`) rather than by the corpus.

---

## 3. Discrimination matrix

Every fix reverted in turn, three test binaries run `--no-fail-fast`. No fix is unguarded and
no test is decorative.

| revert | tests that redden |
|---|---|
| A production-counting | `t1`, `t2`, `t4`, `t5`, `t6`, `t10`, `t20`, `t22` |
| B residual subtraction | `t7`, `t18` |
| C bot tap demotion | `t8`, `t20`, `t21` |
| D layer-resolved sources | `t16`, `test_s8_scripted_human_playthrough_is_clean_on_five_seeds` |
| E summoning sickness | `t11`, `t19` |
| F activation condition | `t15`, `t19` |
| G life cost | `t13` |
| H counter cost | `t14` |
| I ability mana component | `t12` |
| J offer-side predicate | `t19`, `t22` |
| K least-waste selection (phase 3 → first-untapped, main's rule) | `t3` |
| L stax restrictions | `t22` |
| M scaled-ability exclusion | `t23` |
| bot path → old `advance()` body | `t21` |

Two corrections to this table's own method, both from `/review`. (1) The first run used
cargo's default fail-fast and under-reported revert D by one test — the second binary never
ran; re-run with `--no-fail-fast`. (2) The first version had **no row for `pick_least_waste`**
while claiming no test was decorative, which left `t3` — the sole guard for the
"prefer small producers" half of criterion 2 — looking unguarded. Row K exists now and
reddens exactly `t3`.

**Tests that appear in no row, and why that is correct**: `t9` and `t9b` pin *non-vacuity*
of the bot demotion (a scored-0 action stays choosable; a spend target still outranks it),
which is a property of the fix's shape rather than of the fix, and `t17` pins the
`any_color` contract (one mana, colour announced, never colorless) that SIM-2 preserved
rather than changed. Neither is decorative and neither has a revert — they would redden on a
*future* regression, which is what a pin is for.

---

## 4. Fuzzer A/B

`mtg-fuzzer --games 100 --seed 1 --threads 1 --max-turns 60 --verbose`, merge base
(`8cad9c36`) vs this branch, one line per game:

* **96 of 100 games byte-identical** (turns, commands, violations, outcome);
* 4 differ **only** in command count (seeds 15, 71, 73, 82: 2400→2403, 2981→2564, 2399→2460,
  2394→2434);
* violations **0 → 0**; every game ends `MaxTurnsReached(60)` on both sides; total commands
  238,560 → 238,247.

The four are explained by the offer set moving: `can_afford` now credits multi-mana sources
truthfully and withholds unactivatable taps, so `RandomBot`'s uniform pick lands elsewhere.
The fuzzer's default bot is `RandomBot`, which is untouched by the `HeuristicBot` change.

**Long games are a different story, and it is worth stating precisely.** At `--max-turns 200`
the merge base completed 30/30 games across seed bases 500…3000; this branch aborts on
**seed 504** with `fatal runtime error: stack overflow`. That is not a SIM-2 defect: the
recursion cycle contains no simulator frame, the batch changes no engine line, and the cause
is `indomitable_archangel`'s metalcraft static making `calculate_characteristics`
self-referential (**OOS-SIM2-6**, diagnosed by `gdb` backtrace and a depth probe that named
the card). What SIM-2 did was re-roll which games reach that board. It is very likely the
mechanism behind **OOS-M11-3 / OOS-DP3-9**, which recorded the symptom without a cause.

---

## 5. The seed pin, re-derived (second time in two days)

`tools/play-server`'s `TARGET_SEED` moved **1 → 13**. The pin's own comment states the rule
this follows — the pins are a function of the whole corpus *and of the provider*, and any
change to `legal_actions.rs` invalidates them. Method: run the four fixtures against
`seed ∈ 0..24`; **only 13 passes all four**. That is a stricter check than the property sweep
it replaces, because it asserts the fixtures rather than their preconditions.

Seed 1 does not merely miss the fixture now — it drives the engine into an `i32` **overflow
panic** in `layers.rs`'s `ModifyPower` arm (`Devilish Valet` doubling its own power, observed
at `power = delta = 2^30`). Filed as **OOS-SIM2-5**; recorded at the pin so that "seed 1
panics" cannot later read as a property of the fixture.

---

## 6. Numbers

* Tests: `sim2_mana_intelligence` **24**, `sim2_mana_source_roster` 5; `play-server` 40/40;
  simulator suite green; workspace **4,185 → 4,214** (merge base measured in a clean worktree
  at `8cad9c36`).
* PROTOCOL / HASH: gate-executed, unmoved (the criterion's "PROTOCOL 32" was stale —
  PB-DX6 moved it to 33 before this fork).
* Coverage: unmoved — zero card-def edits, zero completeness flips.
* Benches: not run; the changed code is not on a bench path (`priority_cycle_4p` and
  `full_turn_4p` do not enumerate legal actions), and the fuzzer A/B above is the
  representative measurement.

## 7. Seeds filed

`OOS-SIM2-1..7` plus the retro-filed and closed `OOS-CARDS2-9`, all in
`docs/audits/decision-point-audit.md` §8.1. The two out-of-scope engine findings
(**OOS-SIM2-5**, **OOS-SIM2-6**) are the ones worth carrying past this batch: the second is a
hard, unrecoverable crash reachable from a legal deck.
