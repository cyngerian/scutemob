# PB-DX29 — execution notes

**Task**: `scutemob-211` · **Branch**: `feat/pb-dx29-the-paramsrs-allowlist-the-cost-kind-surface-oos-m11`
**Seeds**: `OOS-M11-10(loyalty)` + `OOS-UI2-4`
**Brief**: `memory/primitives/seed-rerank-2026-08-02.md` §4 row 13, plus §1c's two corrections and
the ⚠️ ID-collision note under `docs/audits/decision-point-audit.md` §8.1's table.

> Every number in this file was measured on this branch by an executed command. Where a figure is
> quoted from a brief, a seed row or an in-source comment, it is labelled as such and then
> re-derived — per dispatch hygiene 6, **a brief's site list is a floor, not a census**.

---

## 0. Baseline, measured BEFORE any edit

`~/.cargo/bin/cargo test --workspace --no-fail-fast` to a file, on this branch at `53ecbd36`
with a clean tree:

| | |
|---|---|
| passed | **4,634** |
| failed | **0** |
| ignored | **5** |
| result-producing targets | **46** |
| residual list | empty |

This reproduces PB-DX28's close-out pin exactly (CLAUDE.md's 2026-08-14 delta), so the number is
measured on this branch rather than inherited. The full test-NAME set was captured for the
end-of-batch set-diff (4,639 lines = 4,634 passed + 5 ignored).

---

## 1. Census A — the `params.rs` allowlist and the hard-coded-empty-targets arms

**Method**: read `crates/simulator/src/params.rs` directly and enumerate; the brief's own figures
are treated as snapshots.

### 1.1 The allowlist is **nine** arms, as §1c predicted

`action_to_command_with_params`' guard (`params.rs:270-282`) admits exactly:
`CastSpell`, `ActivateAbility`, `DeclareAttackers`, `DeclareBlockers`, `OrderBlockers`,
`KeepHand`, `DiscardToHandSize`, `ChooseTriggerTargets`, `AnswerEffectChoice`.

`LegalAction::ActivateLoyaltyAbility` is outside it. Announcing `targets` on it is refused with
`ParamError::UnsupportedParam("targets")` → HTTP **400**, and the arm then hard-codes
`targets: Vec::new(), x_value: None` (`params.rs:611-620`). The seed's conclusion holds.

### 1.2 The seed's sibling list is **wrong about one of its three members**

The in-source comment at `params.rs:608-610` says the same shape "applies to `ActivateBloodrush`
and the Mutate/Morph casts below, which also hard-code `targets: Vec::new()`". Measured:

| arm | hard-codes `targets: Vec::new()`? | verdict |
|---|---|---|
| `ActivateLoyaltyAbility` (`:618`) | **yes** | REAL — the batch's subject |
| `CastWithMutate` (`:544`) | yes | real but see §1.3 |
| `CastMorphFaceDown` (`:582`) | yes | **REFUTED as a defect** — CR 708.2a: a face-down spell has no text and therefore no targets. Empty is *correct*, not a gap. |
| `ActivateBloodrush` (`:525-529`) | **no** | **REFUTED as a member.** `Command::ActivateBloodrush` has a scalar `target`, forwarded verbatim from the `LegalAction`, and `legal_actions.rs:1300-1305` emits **one action per attacking creature** — the choice lives in the action list, which is the `PayEcho`/`ChooseDredge` shape PB-DX23 already ruled correct. The comment is wrong; there is no `targets: Vec::new()` on that arm at all. |

A non-test grep for `targets: Vec::new()` in `params.rs` returns exactly three sites — `:544`,
`:582`, `:618` — so the inverse census agrees with the forward one.

### 1.3 `CastWithMutate` — the real lost choice is `on_top`, not `targets`

`params.rs:554-557` hard-codes `AdditionalCost::Mutate { target, on_top: true }`. The *target* is
carried per-action (`legal_actions.rs:1429-1434` emits one action per legal mutate target, the
Bloodrush shape). `on_top` is the caster's choice under CR 702.140a and is not offered anywhere.

### 1.4 The enforcement-site list is short by **three** sites, in two crates

The seed row scopes the closure to "an allowlist arm plus two view-side arms". Measured, the
sites that must agree for a loyalty target to reach the engine are:

| # | site | in the brief? | why it is load-bearing |
|---|---|---|---|
| 1 | `crates/simulator/src/params.rs::action_to_command_with_params` — allowlist + arm | yes | the mapping itself |
| 2 | `tools/play-server/src/view.rs::action_target_requirements` (`:1477`) | yes | the browser picker |
| 3 | `tools/play-server/src/view.rs::target_query_source` (`:1497`) | **no** | `action_option_view`'s `slots` closure returns `Vec::new()` on `None`, so requirements alone render **no candidates** |
| 4 | `crates/simulator/src/targeting.rs::action_target_requirements` + `target_query_source` (`:121`, `:148`) | **no** | the **bot** path. Fixing only the human path leaves every bot's loyalty activation untargeted, which is the SIM-5 defect re-created on a new action |
| 5 | `tools/play-server/src/view.rs::action_needs_x` (`:1389`) | **no** | `LoyaltyCost::MinusX` — without this the browser never renders the X box |

### 1.5 `queries.rs::ability_target_requirements` is the **wrong query** for a loyalty ability

The seed row says "CR 602.2b targets are already reachable through
`queries.rs::ability_target_requirements`' sibling path". That function indexes
`chars.activated_abilities` (`queries.rs:168-172`) — the **layer-resolved activated** list.
`handle_activate_loyalty_ability` indexes a *different* list: `def.abilities` filtered to
`AbilityDefinition::LoyaltyAbility`, read from the **card registry**
(`engine.rs:3605-3616`), and `legal_actions.rs:1228-1236` mints `ability_index` against that same
filtered registry list. The two index spaces are unrelated; reusing the activated query would
have announced a *different ability's* requirements, or none.

### 1.6 A `Complete`, deck-legal card is live-wrong on the `x_value` half

`chandra_flamecaller.rs` declares `completeness: Completeness::Complete` explicitly (`:86`) and
carries `LoyaltyCost::MinusX` (`:74`) with `EffectAmount::XValue` in its effect. The provider
offers the ability unconditionally (`legal_actions.rs:1225`: `MinusX => true, // X can be 0`) and
`params.rs:619` hard-codes `x_value: None` → `x_value.unwrap_or(0)` at `engine.rs:3655`. So the
printed "−X: Chandra Flamecaller deals X damage to each creature" is, for every client in the
tree, **−0 for 0 damage**. `ugin_the_spirit_dragon` has the same shape but is `partial`.

---

## 2. Refusal channel — the "before" measurement (SIM-6 precedent)

Instrument: the existing SIM-5 A/B harness
(`crates/simulator/tests/sim5_bot_cast_discipline.rs::seeded_four_bot_game_wastes_no_taps`,
`-- --nocapture`), seeds **0 / 7 / 42**, 25 turns, four `HeuristicBot`s, journal on. Raw output
retained at `memory/primitives/pb-dx29-refusal-before.txt`.

| seed | rejections (retained/total) |
|---|---|
| 0 | 47 / 47 |
| 7 | 5 / 5 |
| 42 | 53 / 53 |
| **total** | **105** |

Classified:

| class | count |
|---|---|
| `Rejected(InsufficientMana)` on an `ActivateAbility` | 76 |
| `Rejected(InvalidCommand("The attacking player cannot declare blockers"))` | 13 |
| `Rejected(CrossPlayerBlock { .. })` | 14 |
| `Rejected(InvalidTarget("expected 1..=1 target(s) but got 0"))` on an `ActivateAbility` | 2 |
| **additional-cost refusals of any kind** | **0** |

**The zero is the finding, not a null result.** Bots never announce an additional cost at all —
`params.rs`' merge appends the offer's own `default` when the caller announced nothing
(`params.rs:709-760`), so the bot path is structurally incapable of producing a cost refusal.
The cost-kind residue `OOS-UI2-4` names is therefore a **human-path** defect, and this bot A/B is
a *floor* on the batch's effect, not the measurement of it. The human channel is measured
separately, through play-server HTTP probes.

---

## 3. Census B — the 15 `AdditionalCost` variants

**Method**: SR-36. A temporary `#[test]` enumerated `mtg_engine::all_cards()` (**1,803 defs**),
walking the front face, `back_face`, `adventure_face` and nested `ClassLevel` abilities;
deck-legality is `def.completeness.is_complete()`, the exact predicate `validate_deck` uses
(`commander.rs:233`). The temp file was deleted and the tree verified clean afterwards. A second,
independent pass read every consumer of `CastSpellData.additional_costs` / `StackObject.
additional_costs` in `casting.rs`, `resolution.rs`, `copy.rs`, `hash.rs`, `replay_harness.rs`,
`params.rs`, `legal_actions.rs`, `api.rs` and `view.rs`.

**The enum has 15 variants** — memo §1c's correction is confirmed, and Kicker is not one of them
(`CastSpellData.kicker_times`).

### 3.1 The per-kind matrix

`M` = mandatory (omitting it is a hard refusal). `O` = legally optional (omitting it declines).
"reachable" = a client can actually get the engine to that branch today.

| # | variant | deck-legal `Complete` members | M/O | reachable today? | disposition |
|---|---|---|---|---|---|
| 1 | `Sacrifice` | **13** | M (CR 118.8) / O (Bargain, Casualty, Devour) | yes | **CONTROL** — shipped by UI-2 |
| 2 | `Squad` | **2** | O | yes | **CONTROL** — shipped by UI-2 |
| 3 | `Replicate { count }` | **1** (Train of Thought) | O | yes | **BUILD** |
| 4 | `Splice { cards }` | **1** (Glacial Ray) | O | yes | **BUILD** |
| 5 | `Entwine` | **1** (Goblin War Party) | O | yes | **BUILD** |
| 6 | `Fuse` | **2** by marker / 3 by cost | O | yes | **BUILD** |
| 7 | `Gift { opponent }` | **1** (Nocturnal Hunger — see §3.2) | O | yes, after the marker repair | **BUILD** |
| 8 | `EscalateModes { count }` | 0 | O | yes | **BUILD** (same picker shape as Replicate) |
| 9 | `Offspring` | 0 | O | yes | **BUILD** (same picker shape as Entwine/Fuse) |
| 10 | `CollectEvidenceExile { cards }` | 0 | O by default, M per-def flag | yes (the spell is cast from **hand**; only the exiled cards come from the graveyard) | **BUILD** (same picker shape as Splice) |
| 11 | `Assist { player, amount }` | **1** (Huddle Up) | O | yes | **DEFER** — §3.3 |
| 12 | `Mutate { target, on_top }` | **6** | M when `alt_cost == Mutate` | target: yes, per-action. `on_top`: no channel | **PARTIAL / DEFER** — §3.4 |
| 13 | `Discard` (Retrace, Jump-Start) | 1 + 1 | M by two different paths | **no** — §3.5 | **DEFER** |
| 14 | `EscapeExile { cards }` | 0 | M | **no** — §3.5 | **DEFER** |
| 15 | `ExileFromHand { card }` | **4** (Force of Will/Negation/Vigor, Misdirection) | M when `alt_cost == Pitch` | **no** — §3.5 | **DEFER** |

**The seed's "13 of 15 kinds" is arithmetically right and materially misleading.** Four of the
thirteen (`Escape`, `CollectEvidence`, `Escalate`, `Offspring`) have **no deck-legal member at
all**, and three more (`Discard`, `EscapeExile`, `ExileFromHand`) are unreachable by construction
regardless of what the client renders. Counting *variants* measures the enum; counting
*deck-legal reachable members* measures the human's actual loss.

### 3.2 A new live defect: the Squad `R3b` shape, inverted, on a deck-legal `Complete` def

`nocturnal_hunger` is `Completeness::Complete` and deck-legal, carries the cost-bearing
`AbilityDefinition::Gift { GiftType::Food }`, and carries **no `KeywordAbility::Gift` marker**.
`casting.rs:2825` gates on the marker before it looks the cost up, so the printed Gift is
**unpayable** — `"spell does not have gift (CR 702.174a)"`.

This is exactly the defect UI-2's `r3b_squad_marker_and_squad_cost_declare_the_same_defs` was
written for, one enum variant over, and it survived because that gate is Squad-specific. Two more
defs carry the same disagreement:

| def | kind | shape | completeness |
|---|---|---|---|
| `nocturnal_hunger` | Gift | cost, **no marker** | **`Complete`, deck-legal** |
| `connive_concoct` | Fuse | cost, **no marker** | `Complete`, deck-legal |
| `tooth_and_nail` | Entwine | **marker, no cost** | `partial` |
| `dawns_truce` | Gift | marker, no cost | `partial` |

### 3.3 `Assist` is deferred because building it would ship a worse bug than the one it fixes

CR 702.132a requires **another player** to choose to activate mana abilities and pay.
`casting.rs:3894-3948` runs entirely inside the caster's `handle_cast_spell`: it validates that
the named seat is another active player and then **unilaterally drains that player's mana pool**
(`pay_cost(&mut assist_player_state.mana_pool, ..)` at `:3937`), emitting
`ManaCostPaid { player: assist_pid }`. There is no `Command`, no `PendingDecision`, no
`EffectChoiceQuestion` and no `LegalAction` involving the assisting seat anywhere in the tree.

Rendering an Assist picker today would hand a human a button that spends an opponent's floating
mana without asking them. The channel that is missing is not a picker; it is a cross-seat
decision, which is engine + wire work. Filed rather than built.

### 3.4 `Mutate.on_top` is deferred because the engine asks at the wrong time

CR 702.140c: *"the spell's controller chooses whether the spell is put on top of the creature or
on the bottom"* — **as the spell resolves**. The engine captures the bit in the `AdditionalCost`
at announcement (`casting.rs:133-136`, bound to `_mutate_on_top` and deliberately unused there),
carries it on the `StackObject`, and reads it at `resolution.rs:7377-7384`. Adding a cast-time
picker would make a CR-wrong timing *more* visible and more load-bearing (CR 702.140e: the
topmost card supplies the permanent's non-ability characteristics), so the correct fix is a
resolution-time `EffectChoiceQuestion` — the PB-DX28 suspend-and-replay channel — not a cast-time
box. Filed.

### 3.5 Three kinds are unreachable by construction, and no picker changes that

`Discard` (Retrace / Jump-Start), `EscapeExile` and `ExileFromHand` all require a cast the client
cannot currently make:

* `StubProvider`'s two cast loops walk `ZoneId::Hand(player)` (`legal_actions.rs:774`) and
  `ZoneId::Command(player)` (`:871`) and **no graveyard**, so Retrace, Jump-Start, Escape and
  Flashback casts are never offered at all;
* `params.rs:306` hard-codes `alt_cost: None` on every `CastSpell`, so the Pitch branch
  (`casting.rs:4213`) is unreachable and all four Force-of-Will-shaped defs are unplayable at
  their pitch cost from any client.

Building pickers for these would produce code that cannot be verified end to end with a
non-default answer — the project's own acceptance standard since UI-4 — so they are deferred with
the enabling work named (a graveyard cast loop and an `alt_cost` channel) rather than half-built.

---

## 4. What shipped

### 4.1 Part A — the loyalty channel (`OOS-M11-10(loyalty)`)

| link | change |
|---|---|
| `crates/engine/src/rules/queries.rs` | **new** `loyalty_ability_target_requirements` + `loyalty_ability_needs_x`, re-exported from `lib.rs` |
| `crates/simulator/src/params.rs` | `ActivateLoyaltyAbility` joins the allowlist (nine arms → **ten**); the arm forwards `targets` and `x_value` |
| `crates/simulator/src/targeting.rs` | loyalty arms in `target_query_source` **and** `action_target_requirements` — the **bot** path |
| `crates/simulator/src/legal_actions.rs` | **new** `loyalty_ability_is_offerable` — SR-38 suppression of an offer whose mandatory slot has no candidate |
| `tools/play-server/src/view.rs` | loyalty arms in `action_target_requirements`, `target_query_source` **and** `action_needs_x` |
| `tools/play-server/frontend/` | **zero production lines** — the `'value'` and `'targets'` stages are gated on `option.needs_x` / the slot list, not on the action kind |

**`x_value: 0` maps to `None`, not `Some(0)`.** The engine reads `x_value.unwrap_or(0)`, so the two
are behaviourally identical to it — but `Command` is serialized into the replay log and the
journal, so `Some(0)` would change every bot-driven loyalty activation's recorded bytes for no
behavioural gain. This mapping keeps a default-params bot producing the byte-identical
pre-PB-DX29 command, and no recorded seed moves.

### 4.2 Part B — the cost-kind surface (`OOS-UI2-4`)

Seven kinds added across the seven-link chain (provider → view DTO → picker → stage → POST →
400 gate → params → engine): **Replicate, EscalateModes, Entwine, Fuse, Offspring, Gift, Splice**.

**The link the seed row does not name is the one that mattered.**
`legal_actions::effective_cast_cost_with_additional` read **Squad and nothing else**, and it is
the function `LocalGame::auto_tap_commands_for` asks how much mana to tap. Shipping the pickers
without extending it would have tapped the base cost, accepted the human's announcement, and let
the engine refuse the whole cast with `InsufficientMana` — **this batch would have created the
exact SR-38 defect it was dispatched to remove.** Every arm now mirrors `casting.rs` clause for
clause: last-wins for the scalars (its destructuring loop is a plain assignment), a real sum for
Splice (its own loop is), the seven numeric components only, and nothing at all for Gift.

`squad_max_count` → `repeated_cost_max_count`, closing UI-2's asymmetry: it built an
affordability bound for Squad and nothing analogous for the structurally identical Replicate.

### 4.3 Card-def repairs — three, all comment-and-declaration, **0 completeness flips**

| def | repair | completeness |
|---|---|---|
| `nocturnal_hunger` | added the missing `KeywordAbility::Gift` marker | `Complete` before and after — **the repair flips nothing because the def was already deck-legal and already wrong** |
| `tooth_and_nail` | authored the printed `Entwine {2}` (was marker-only) | stays `partial` on its unrelated "up to two" search blocker |
| `dawns_truce` | authored `AbilityDefinition::Gift { GiftType::Card }` (was marker-only) | stays `partial`; its note corrected |

`connive_concoct` was flagged by the census and is **REFUTED by its own in-source comment**: it
carries `AbilityDefinition::Fuse` without the marker *deliberately*, as the data carrier for a
split card's right half. Recorded as a named exception in the new gate rather than "fixed".

---

## 5. What the machine gates caught, and what that cost

Four independent gates fired on this batch's own work. Every one was right.

| gate | what it caught |
|---|---|
| **SR-5 keyword registry** | seven keywords gained `crates/simulator/src/legal_actions.rs` as a handling site. The same gate caught PB-DX20 (`queries.rs` is an Enchant site) and PB-DX23 (`queries.rs` is a Dredge site); this is its third consecutive catch. |
| **`ability_definition_registry`** | seven `AbilityDefinition` variants gained the same site, **plus `A::LoyaltyAbility` gaining `rules/queries.rs`** — which the keyword gate could not see, because a loyalty ability is not a keyword. |
| **`pb_dx27_stale_blocker_notes`** | **this batch's own `dawns_truce` note.** The rewrite moved its phrasing from OUTSIDE the `GAP_NEEDLES` vocabulary ("unimplemented") to inside it ("not expressible"), so a note that had always named live identifiers became newly visible and the ratchet went 107 → 108. Reworded to restore the prior classification honestly, with the possibility that the def is now **authorable** recorded in the note itself (`OOS-DX29-7`) rather than resolved by a phrasing change. |
| **`pb_dx29_additional_cost_roster` R3** (this batch's own new gate) | `brokkos_apex_of_forever`'s `{2}{G}{G}{U/B}` mutate cost, against a formatter that rendered neither hybrid nor Phyrexian pips. See §6. |

### 5.1 The gate that found the thing its own author's predecessor promised it would

UI-2 wrote `ui2_additional_cost_roster::r4` asserting that no def in the corpus has a hybrid or
Phyrexian **Squad** cost, because `view.rs::format_mana_cost_compact` rendered neither and such a
cost would display as strictly cheaper than it is. Its comment promised the gate would "fail
loudly the day one is authored".

PB-DX29's R3 is that assertion widened past Squad, and it went red on its **first run** — on
`brokkos_apex_of_forever`, a counter-example the corpus had carried the whole time. The day had
already arrived, on a different cost kind, and the Squad-scoped gate could not see it.

**A gate written for one variant measures that variant.** That is this batch's thesis (it is why
R2 exists at all, after `nocturnal_hunger` reproduced `galadhrim_brigade`'s defect one enum
variant over) and it arrived a second time inside the batch's own work. The fix is the formatter
— it now renders CR 107.4e hybrid, CR 107.4f Phyrexian and CR 107.3 `{X}` — not a narrower gate,
and UI-2's own R4 doc and failure message are corrected in place rather than left asserting a
limitation that no longer exists.

### 5.2 Two defects in part A, found by the batch's own test author

Both were in code committed before the tests were written, both were reported rather than worked
around, and both were taken.

**F1 (MEDIUM) — the new queries panicked in debug, contradicting their own rustdoc.** Both used
`GameState::expect_object`, the *impossible-absence* lookup (`state::diagnostics`), which fires a
`debug_assert!` and degrades to `None` only in release. Their doc promised "never panics";
`queries.rs`'s module doc calls the whole file a read-only **advisory** surface for UI callers;
and every other lookup in that file avoids it. **What is impossible for an engine-internal caller
is ordinary input for a UI one** — a CR 400.7-retired id from a stale browser is not an engine
bug. Fixed to `state.objects().get(&source)`. The test was pinned wrong-way-round and is now
inverted to assert the contract in **both** profiles.

**F2 (LOW) — the fix widened a declared residual and the doc recording it went stale in the same
commit.** Joining the allowlist is what makes `first_announced_field` stop running for an arm, so
`ActionParams { attackers, .. }` on an `ActivateLoyaltyAbility` was a loud
`UnsupportedParam("attackers")` before and is an `Ok` with the field dropped after. The trade is
right — the alternative was refusing the `targets` and `x_value` that arm now genuinely reads —
but it moves `params.rs`' own "nine consuming arms" residual to **ten**, and that doc still said
nine. Corrected, and filed as `OOS-DX29-8`.

### 5.3 A gate's WALK is narrower than its CLAIM — the third instance in one batch

Found by the frontend author while wiring the picker, not by any test.

`ui2_additional_cost_roster::r5` justifies `ActionBar.svelte`'s stage order with the claim *"no
def declares an additional cost together with `{X}` or modes"* — and its predicate walks
`spell_additional_costs` and Squad **only**. **Escalate (CR 702.120a) and Entwine (CR 702.42a)
are additional costs on modal spells by definition** — `casting.rs` requires a modal spell for
escalate in so many words, and entwine's whole function is "choose all modes". So R5 reports a
clean board while the condition it was written to detect is live on **five** corpus defs.

That is the same shape as `r3b` staying green while `nocturnal_hunger` was broken, and as UI-2's
R4 staying green while `brokkos_apex_of_forever` sat in the corpus. Three instances, one batch.

The replacement (`pb_dx29_additional_cost_roster::R6`) **prints** the offenders instead of
asserting their absence, because their absence is not true and asserting it would be a lie that
happens to pass. What it asserts is the half that actually matters to the client:

| measured | value |
|---|---|
| modal additional-cost defs | **5** — `Goblin War Party`, `Promise of Power`, `Tooth and Nail` (Entwine); `Blessed Alliance`, `Collective Resistance` (Escalate) |
| additional-cost defs carrying an `{X}` | **0** |

So the stage-order inversion is modes-vs-costs — which is CR 601.2b's *own* order, and therefore
harmless — and never `{X}`-vs-costs, which is the half that would be wrong. R5 is kept unchanged
with its narrowness stated at the test, rather than widened into a failure.

---

## 6. The refusal channel, after — and the honest reading of an unmoved number

Re-run of the identical instrument on the identical seeds after every change:

| | before | after |
|---|---|---|
| seed 0 | 47 | 47 |
| seed 7 | 5 | 5 |
| seed 42 | 53 | 53 |
| **total** | **105** | **105** |

`diff` over the classified breakdowns is **empty**. Raw output:
`memory/primitives/pb-dx29-refusal-before.txt` / `-after.txt`.

**What that does and does not prove.** It does *not* prove the batch fixed nothing; it proves the
batch is **bot-path-neutral**, which is the property that keeps every recorded seed alive. Bots
never announce an additional cost (`params.rs` appends the offer's own default and nothing else),
and in these three seeds no bot ever activates a targeted loyalty ability — so neither half of
this batch has a bot-path expression, and the 105 are the same 76 `InsufficientMana`, 13
"attacking player cannot declare blockers", 14 `CrossPlayerBlock` and 2 `InvalidTarget` as before.

**The channel that moved is the human one**, and it is measured separately, through the
play-server HTTP probes: refusals that used to be a bare **422** from the engine (or, worse, a
silent loss of an announced rider) are now either a **400** naming the offer the answer
contradicts, or an accepted answer with an observable game-state effect. Publishing the unmoved
105 alongside that is the point — an A/B on the wrong channel that came back flat would otherwise
read as evidence of nothing happening.
