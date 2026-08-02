# First-human-playtest triage — 2026-08-02

Source: `test-data/bot testing notes.md` (user's notes from the first extended human
playtest of the M11-local browser client, 2026-08-01/02). Every functional claim was
verified against code on main `c49986f6`/`e4b93ac0` by two read-only investigation
passes before any task was filed. This file is the durable evidence record; the ESM
tasks (scutemob-174..181) reference it. Line cites below are snapshots (OOS-DP6-8
class) — re-verify by symbol before building on them.

## Headline

Every confirmed defect lives in `crates/simulator`, `tools/play-server`, or card defs.
**Zero engine bugs.** The engine's payment chain was traced end-to-end specifically
(colored pips cannot be paid with generic mana anywhere; `flatten_hybrid_phyrexian`
only converts hybrid/Phyrexian *into* colored; `ManaPool::can_spend`/`spend` check
colored buckets before generic).

## Verified findings

### F1 — Boon Satyr castable with 1 green: the DEF is wrong, not the engine
**CLOSED 2026-08-02 by CARDS-2 (`scutemob-181`).** All four defects repaired, including the
"+4/+2" that was never authored — it is two layer-7c statics on `EffectFilter::AttachedCreature`,
the shape `rancor.rs` already used. Proven by execution, not asserted: reverting the statics
leaves the enchanted 2/2 at 2/2 (`primitives::cards2_printed_field_repair::t5`). Evidence:
`memory/card-authoring/cards2-field-fidelity-2026-08-02.md` §2.
`crates/card-defs/src/defs/boon_satyr.rs:9-13` transposes the pips: `generic: 2,
green: 1` (= `{2}{G}`) for a printed **{1}{G}{G}**. The engine charged exactly what
the def said. Three more defects in the same file: bestow cost `{4}{G}{G}` vs printed
`{3}{G}{G}` (`:26-29`); missing Enchantment type (`:16` uses `creature_types`); the
"+4/+2 to enchanted creature" continuous effect is not authored at all — yet the def
declares `Completeness::Complete` (`:41`). The engine's own bestow test hardcodes the
correct numbers (`crates/engine/tests/mechanics_a_d/bestow.rs:56-57`).

### F2 — Corpus-wide mana-cost audit: 17 wrong costs / 1,804 defs, 9 deck-legal
**CLOSED 2026-08-02 by CARDS-2 (`scutemob-181`).** All 17 repaired and the audit made permanent
as **SR-37** — `tools/card-field-dump` + `tools/refresh-card-fidelity-fixture.py` +
`core::cards2_printed_field_fidelity`, with a committed fixture so it runs in CI without
`cards.sqlite`. This table was **reproduced exactly, card for card**, by an independent
measurement before anything was repaired. The P/T and type-line extension this finding predicted
was also run: **39 real defects total** (17 cost / 5 P/T / 16 type line / 1 duplicate card name).
Evidence: `memory/card-authoring/cards2-field-fidelity-2026-08-02.md`.
Method: throwaway `all_cards()` dump (name, completeness, `{:?}` of `mana_cost`)
diffed against `cards.sqlite` (Scryfall), face-0 costs for `//` names, non-game
layouts filtered (`token`, `art_series`, …). 1,802/1,804 matched; the 2 unmatched are
documented synthetic test cards (`poisonous_viper`, `steel_guardian`). Self-check:
`radstorm` + `metastatic_evangel` (costs repaired by PB-DX4) correctly absent.
Scripts preserved in the 2026-08-02 session scratchpad (`cost_audit2.py`,
`dump_costs.rs`); trivially recreatable.

**Complete (deck-legal), 9:**

| card | def | printed |
|---|---|---|
| tyrranax_rex | `{G}{G}{G}{G}` | `{4}{G}{G}{G}` — **3 cheap on a 7-drop** |
| boon_satyr | `{2}{G}` | `{1}{G}{G}` |
| braided_net | `{2}` | `{2}{U}` — colored pip dropped |
| necron_deathmark | `{3}{B}` | `{3}{B}{B}` |
| cyber_conversion | `{2}{U}` | `{U}{U}` |
| changeling_hero | `{3}{W}` | `{4}{W}` |
| backup_agent | `{2}{W}` | `{1}{W}` |
| saw_it_coming | `{2}{U}{U}` | `{1}{U}{U}` |
| exalted_angel | `{3}{W}{W}{W}` | `{4}{W}{W}` (same MV, pips shuffled) |

**Non-Complete, 8:** brainsurge `{3}{U}`→`{2}{U}`; flare_of_malice `{3}{B}`→`{2}{B}{B}`;
stock_up `{3}{U}`→`{2}{U}`; glacierwood_siege `{3}{G}`→`{1}{U}{G}`; and a coherent
**dropped-{X} class** (`x_count: 0` where printed has `{X}`): chord_of_calling,
green_suns_zenith, torment_of_hailfire, wake_the_dead — intersects OOS-M11-8's
`x_count` roster.

Errors run in both directions → authoring-time transcription noise at a ~1% background
rate, not one systematic cause. Nothing gates the field; mana cost is the most
mechanically checkable field in a def. Same technique extends to P/T and type lines
(metastatic_evangel precedent: transposed P/T + missing subtype).

### F3 — Auto-tap is all-or-nothing; floating mana wasted
**CLOSED 2026-08-02 by SIM-2 (`scutemob-176`).** `mana_solver::solve_mana_payment_with_pool`
subtracts the pool in `ManaPool::can_spend`'s own order and solves the residual, so the
all-or-nothing check is not repaired but *subsumed*: a fully-covering pool is the
residual-is-zero case of the general rule. The bot path in `advance()` calls the same helper
(and so inherits the commander tax and the announced `{X}`, which it never had — see
`OOS-M11-8`). `can_afford` asks the same question once, closing a third gap this finding did
not name: a player with `{G}` floating and one Forest up was told a `{1}{G}` spell was
uncastable, because neither the pool shortcut nor the whole-cost solve covered the middle.
Evidence: `memory/primitives/sim2-mana-intelligence-2026-08-02.md`.
`crates/simulator/src/local_game.rs:562-578`: if the pool fully covers the flattened
cost → tap nothing (`:574-575`); anything less → `solve_mana_payment` is handed the
**entire printed cost** (`:577`), pool never subtracted. 2 floating + 3-CC cast = 5
tapped, float destroyed at the step boundary (`rules/turn_actions.rs:1581`, CR
500.4/500.5). The gap is admitted in a comment (`local_game.rs:541-543`) but filed
under no seed — OOS-M11-2's two halves both name something else. Existing test covers
only the two extremes (`crates/simulator/tests/local_game.rs:1878-1892`). Fix:
residual cost (flat_cost minus pool) at `:577`; subsumes the `:574` early return.
Bot path (`local_game.rs:427-446`, no pool check by design per `:545-548`) should use
the residual solver once it exists — its "harmless asymmetry" rationale only held
because of this bug.

### F4 — Mana solver counts SOURCES, not MANA; no ordering preference
**CLOSED 2026-08-02 by SIM-2 (`scutemob-176`).** A tapped source now credits its whole
production to a running tally and each pip is paid from that tally, so a multi-mana source's
surplus is spendable on the rest of the same payment. Phase 3 picks by least waste (the
largest production that fits, else the smallest), which is what "prefer small producers"
is actually for: a `{1}` takes the Forest and leaves the Sol Ring up, and a `{2}` takes the
Sol Ring alone rather than two Forests. The four fresh reproductions this section records
below are covered by the same fix, together with **OOS-CARDS2-9**, which named three more
symptoms of the identical defect and was never filed. Evidence:
`memory/primitives/sim2-mana-intelligence-2026-08-02.md`.
`crates/simulator/src/mana_solver.rs:23-147`. `produces` is expanded per unit of mana
(`:39-44`) and the length is then never used: every phase decrements `remaining` by 1
per source tapped. Sol Ring (`{C}{C}`) is credited as 1. Phase 3 (generic, `:122-144`)
picks *the first untapped source in battlefield order* — no heuristic — which is how
Sol Ring + 2 Forests got tapped for `{2}{G}` (4 in pool, 3 spent, 1 stranded).
**Under-offering dual:** a `{2}` spell with only a Sol Ring untapped is judged
unaffordable (`:126` returns None → `legal_actions.rs:1436` suppresses the offer) —
the pool-total shortcut (`legal_actions.rs:1424-1430`) only rescues already-floating
mana. Doc comment (`mana_solver.rs:1-5`) describes the intended behaviour as if
correct; no seed anywhere. Fix: decrement by actual produced count; Phase 3 prefer
ascending `produces.len()`; Phase 1 same credit fix for multi-mana colored sources.

### F5 — Bots tap out every empty upkeep: systematic, not random noise
**CLOSED 2026-08-02 by SIM-2 (`scutemob-176`).** `HeuristicBot` scores `TapForMana` **0**,
below `PassPriority`. The choice between demoting and gating on a spend target is
observationally empty: every mana-consuming action already outscored the old 5, so a tap was
only ever chosen when it was the sole alternative to passing. Scored 0 rather than removed,
so it stays choosable when it is all there is. `RandomBot` is untouched, so no recorded fuzz
seed moves for this reason. Evidence:
`memory/primitives/sim2-mana-intelligence-2026-08-02.md`.
`heuristic_bot.rs:70` scores `TapForMana` **5** vs `PassPriority` **1**
(`heuristic_bot.rs:76`), no spend-intent check → with nothing to cast, the bot
deterministically taps every source, pool wiped at the step boundary, arrives at main
phase tapped out. Header comment (`heuristic_bot.rs:8`) says "prep" but nothing
conditions it. RandomBot's version is expected uniform-choice behaviour
(`random_bot.rs:53-54`). Fix: score TapForMana below PassPriority (LocalGame auto-tap
already funds bot casts), or gate the +5 on an affordable follow-up cast existing.

### F6 — `stack_consistency` invariant is a false positive BY CONSTRUCTION
**CLOSED 2026-08-02 by SIM-3 (`scutemob-177`).** The diagnosis below is exactly right and
M11-local S8 had already acted on it; SIM-3 finished the job. What remained, and what it
found: the check had **no test module** (now ten probes, each watched failing under a
deliberate revert — a 9-revert matrix); `docs/mtg-engine-simulator.md` **and**
`docs/mtg-engine-runtime-integrity.md` both still stated the wrong invariant in prose (S8
corrected neither); and the S8 rewrite carried a **residual false positive of its own** —
it classified on `StackObjectKind::Spell` alone, on the stated premise that every
Stack-zone move "ends in that same `Spell` kind", but `casting.rs` moves the card first
and *then* branches on `cast_with_mutate`, so every **mutate** cast (CR 702.140a) had a
card in the Stack zone that no `Spell` object claimed. Classification is now an exhaustive
match over all 27 `StackObjectKind` variants, so the next variant is a compile error until
someone classifies it. Measured A/B, same builds and seeds: `--games 5 --seed 1
--max-turns 200` goes **9,719 → 938** violations (the 8,781 that vanish are 90.3% of the
run) and the scripted playthrough's seed 1 goes **720 → 0**. Seeds `OOS-SIM3-1..4` filed.

`crates/simulator/src/invariants.rs:100-128` compares `ZoneId::Stack` object ids
against `state.stack_objects()[].id` — two **deliberately different** id namespaces:
`casting.rs:4399` mints the zone object (CR 400.7), `:4401` mints `stack_entry_id =
new_card_id + 1` for the `StackObject`, which is never inserted into any zone
(independently documented at `memory/m11-session-plan.md:792-796`). Every spell on the
stack at check time = 2 violations per checkpoint; the user's 436 ≈ 218 checkpoints
with one spell up. `turn_number: 1` is genuine (first violations were on turn 1; the
UI joins all 436 into one string, user quoted the head). `invariants.rs` has **no test
module**. **Distinct from OOS-DP3-9 / OOS-M11-3** (long-game nondeterminism /
overflow) — and very plausibly the bulk of the "70,719 violations" baseline noise in
those A/B notes; fixing it de-noises that work. Docs stating the wrong invariant in
prose: `docs/mtg-engine-simulator.md:226`, `docs/mtg-engine-runtime-integrity.md:58`.

### F7 — Commander cannot be cast: provider never scans the command zone
Engine fully supports it (`casting.rs:255` command-zone detection, `:4702` tax,
`:4913` event; `setup.rs:258` puts the commander there). `legal_actions.rs:489-492`
enumerates casts **from hand only**; `ZoneId::Command` appears in the file only for
the CR 903.9a return choice (`:339-340`). Frontend is innocent — clicking the
commander correctly reports "the server offered none". Fix is simulator-only: scan
`ZoneId::Command(player)` for `commander_ids`, same gates + **tax-aware
affordability** (2 × cast count, else it 422s). `params.rs:180` already forwards the
bare card; engine derives zone-ness itself. No wire, no engine, no frontend change.

### F8 — Blocking-decision defaults ride inside the LegalAction; client can only echo them
Shared mechanism behind the discard + scry/search symptoms. `StubProvider` bakes the
engine-accepted default into the action (`legal_actions.rs:271-278` cleanup discard:
`cards = default_cleanup_discard` = **highest-ObjectId N**, i.e. "last cards on the
right", exactly as observed — `rules/turn_actions.rs:1404-1419`; `:314-323` effect
choice: `answer = default_effect_choice_answer` — scry/surveil = identity/no-op,
search = `candidates.first()` = lowest ObjectId, both exactly as observed;
`effects/mod.rs:386-395`). `view.rs:135-183` `ActionOptionView` strips the payload;
`ActionParamsDto` (`view.rs:360-381`) has no override channel; `params.rs:166-176`
allowlists only 5 param variants; frontend `ActionBar.svelte:180-192` finds no picker
stage → submits `{}` → default applied. "The button" = `view.rs:664-666` "Answer X's
choice". The candidate data exists at the provider precisely so a client can render a
picker (`legal_actions.rs:145-156` doc) and is thrown away at the view layer. Seeds
already open: **OOS-DP7-6** (discard picker, filed against the TUI), **OOS-DP9-1/7**
(scry picker "the single most valuable S7 widget"), and **OOS-DP8-2**
(`ChooseTriggerTargets`, identical gap, next to bite).

### F9 — Additional costs: request wire EXISTS, offer + provider blind
**CLOSED 2026-08-02 by UI-2 (`scutemob-178`) — for the two cost kinds it names.**
`StubProvider` now builds an `AdditionalCostPlan` on every `CastSpell`, and a required
sacrifice with **no eligible permanent suppresses the whole offer** (SR-38 restored).
`ActionOptionView.costs` carries the descriptor; `CostPicker.svelte` renders it between
`ValuePrompt` and `TargetPicker`. Four HTTP probes drive it end to end (Life's Legacy to
a **non-default** sacrifice; Squad declined; Squad paid twice → three permanents and an
empty pool; the SR-38 suppression, two-sided). `galadhrim_brigade` — the very card below
— also needed a **card-def repair**: it shipped `Complete` carrying
`KeywordAbility::Squad` with **no `AbilityDefinition::Squad { cost }`**, so
`casting.rs::get_squad_cost` returned `None` and every non-zero count was refused;
`core::ui2_additional_cost_roster` R3b now pins that the marker set and the cost set are
the same set. **Read the closure as covering `Sacrifice` and `Squad` only** — the other
fourteen `AdditionalCost` variants are still invisible to the offer (**OOS-UI2-4**).
Original finding follows.

`CastSpellData.additional_costs` (`command.rs:761-765`) covers Sacrifice/Squad/etc.;
`ActionParams.additional_costs` exists and is forwarded (`params.rs:42`, `:206`);
`ActionParamsDto.additional_costs` deserializes (`view.rs:373`). A hand-crafted POST
could pay a sacrifice **today**. Missing: (a) `ActionOptionView` has no field telling
the client a cost is required/available or what's eligible; (b) the provider is
entirely unaware — zero hits for `spell_additional_costs`/`Squad` in
`legal_actions.rs`/bots. Consequences: Life's Legacy (`lifes_legacy.rs:26`) offered
on mana alone then rejected by `casting.rs:3311-3316` — the observed 422, an SR-38
violation (never offer what the engine rejects); Galadhrim Brigade casts fine at
`count: 0`, Squad silently lost (CR 702.157, optional). Fix: provider reads
`spell_additional_costs` + Squad keyword, offer descriptor, CostPicker stage **before**
TargetPicker (CR 601.2b/h precedes 601.2c). No engine/wire-type change.

### F10 — Equip silent fizzle (filed: **OOS-M11-10**, `e4b93ac0`)
**CLOSED 2026-08-02 by CARDS-1 (`scutemob-179`).**
16 of 17 real equip activations declare `targets: vec![]` against a
`DeclaredTarget { index: 0 }` effect; picker never asks; attach fizzles silently.
10 of 16 `Complete`, all via the `#[default]` derive. See the seed row in
`docs/audits/decision-point-audit.md` §8.1 for the full roster and chain.

## Status (added 2026-08-02)

**CLOSED**: F1, F2 (CARDS-2, `scutemob-181`); F3, F4, F5 (SIM-2, `scutemob-176`); F7 (SIM-1,
`scutemob-175`); F8 (UI-1, `scutemob-174`); F9 (UI-2, `scutemob-178`, for `Sacrifice` + `Squad` —
see that entry's scope note and **OOS-UI2-4**); F10 (CARDS-1, `scutemob-179`); F6 (SIM-3,
`scutemob-177`). **OPEN**: none.

> F7's banner was already stale when SIM-2 read this file — SIM-1 closed it on 2026-08-02 and
> only the roll-up went unupdated. Corrected here rather than left, since a roll-up that is
> wrong about one finding is not trusted about the others.

> **Collect-time note (coordinator, 2026-08-02)**: SIM-2 and UI-2 ran in parallel and each
> roll-up was blind to the other; this is the union. UI-2's paragraph below describes the
> one-mana-per-source solver (F4) as still open — that was true on its fork, and SIM-2 closed
> it the same day. OOS-UI2-3's pinned "wrong 0" expectation was re-checked against the merged
> tree at collect (see the combined-tree test run).

**F4 gained a much bigger reproduction in UI-2**, and it is worth reading before any
future batch leans on a fuzz A/B: `mana_solver`'s Phase 3 pays **one generic pip per
SOURCE tapped**, regardless of how much that source produces, so a Sol Ring counts as
one mana. That is what caps `squad_max_count` (**OOS-UI2-3**). Separately and worse,
UI-2 found that **`mtg-fuzzer` has never cast a spell at all** — it never shuffles its
libraries and `random_deck` appends the basics last, so 5 games × 80 turns produced
25,964 hand-card observations and zero non-lands (**OOS-UI2-1**). Every "fuzz parity"
claim in this project's history is a claim about a land-only game.

F4 gained four fresh reproductions in CARDS-2: sweeping `seed` ∈ 0..24 for a play-server test
fixture found that seeds 2, 10, 11 and 17 are each **offered** a targeted cast and then refused
by the engine with "player does not have enough mana to pay the cost" once sources are tapped.
Whoever fixes the mana solver has four ready cases.

CARDS-2 also filed **OOS-CARDS2-4**, a *new* member of the same SR-38 family as F4 and F9: an
**Aura** is offered with `target_min: 0`, because its target requirement lives in
`KeywordAbility::Enchant(...)` which `casting.rs:3720` special-cases (CR 303.4a) and the provider
never reads — so a human clicking any Aura in the browser client gets a 422.

## Not verified (by design)
UX/layout requests from the notes (hover preview, 3-tier event log, combat display,
2x2 battlefields, sticky player cards, hand bar, dead-player collapse, pass-until-X,
target-selector grouping, deck synergy) are feature work, not claims — carried
directly into the UX task. Deck-synergy quality and deeper bot play strength are
deferred to the M12 agent-authoring track.
