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
