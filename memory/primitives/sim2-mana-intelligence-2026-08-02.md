# SIM-2 — mana intelligence (`scutemob-176`, 2026-08-02)

Evidence record for the batch that closed playtest-triage **F3**, **F4** and **F5**
(`memory/playtest-triage-2026-08-02.md`), plus **OOS-CARDS2-9** (never filed, only named in
source), the layer half of **OOS-M11-2**, and the bot half of **OOS-M11-8**.

Scope as dispatched: `crates/simulator` only. Actual diff: `crates/simulator` (3 source
files, 2 test files) + `tools/play-server/src/main.rs` (one seed pin) + this file + the seed
rows. **Zero engine lines** — `git diff main -- crates/engine crates/card-types
crates/card-defs` is empty.

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

Pinned by `crates/simulator/tests/sim2_mana_source_roster.rs`, over 1,803 defs:

| row | population | count |
|---|---|---|
| — | defs with any `{T}` mana ability | 322 |
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
| A production-counting | `t1`, `t2`, `t4`, `t5`, `t6`, `t10`, `t20` |
| B residual subtraction | `t7`, `t18` |
| C bot tap demotion | `t8`, `t20` |
| D layer-resolved sources | `t16`, `test_s8_scripted_human_playthrough_is_clean_on_five_seeds` |
| E summoning sickness | `t11`, `t19` |
| F activation condition | `t15`, `t19` |
| G life cost | `t13` |
| H counter cost | `t14` |
| I ability mana component | `t12` |
| J offer-side predicate | `t19` |
| bot path → old `advance()` body | `t21` |

(The first run of this matrix used cargo's default fail-fast and under-reported revert D by
one test — the second binary never ran. Re-run with `--no-fail-fast`; the table above is the
complete one.)

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

* Tests: `sim2_mana_intelligence` 21, `sim2_mana_source_roster` 5; `play-server` 40/40;
  simulator suite green; workspace green.
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
